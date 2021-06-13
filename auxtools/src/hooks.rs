use super::proc::Proc;
use super::raw_types;
use super::value::Value;
use crate::runtime::DMResult;
use boomphf::{Mphf, hashmap::NoKeyBoomHashMap};
use detour::RawDetour;
use std::ffi::c_void;
use std::ffi::CStr;
use std::os::raw::c_char;

#[doc(hidden)]
pub struct CompileTimeHook {
	pub proc_path: &'static str,
	pub hook: ProcHook,
}

impl CompileTimeHook {
	pub fn new(proc_path: &'static str, hook: ProcHook) -> Self {
		CompileTimeHook { proc_path, hook }
	}
}

inventory::collect!(CompileTimeHook);

// TODO: This is super deceptively named
#[doc(hidden)]
pub struct RuntimeHook(pub fn(&str));
inventory::collect!(RuntimeHook);

extern "C" {
	static mut call_proc_by_id_original: *const c_void;

	static mut runtime_original: *const c_void;
	fn runtime_hook(error: *const c_char);

	fn call_proc_by_id_hook_trampoline(
		usr: raw_types::values::Value,
		proc_type: u32,
		proc_id: raw_types::procs::ProcId,
		unk_0: u32,
		src: raw_types::values::Value,
		args: *mut raw_types::values::Value,
		args_count_l: usize,
		unk_1: u32,
		unk_2: u32,
	) -> raw_types::values::Value;
}

pub enum HookFailure {
	NotInitialized,
	ProcNotFound,
	AlreadyHooked,
	UnknownFailure,
}

impl std::fmt::Debug for HookFailure {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::NotInitialized => write!(f, "Library not initialized"),
			Self::ProcNotFound => write!(f, "Proc not found"),
			Self::AlreadyHooked => write!(f, "Proc is already hooked"),
			Self::UnknownFailure => write!(f, "Unknown failure"),
		}
	}
}

pub fn init() -> Result<(), String> {
	unsafe {
		let runtime_hook = RawDetour::new(
			raw_types::funcs::runtime_byond as *const (),
			runtime_hook as *const (),
		)
		.unwrap();

		runtime_hook.enable().unwrap();
		runtime_original = std::mem::transmute(runtime_hook.trampoline());
		std::mem::forget(runtime_hook);

		let call_hook = RawDetour::new(
			raw_types::funcs::call_proc_by_id_byond as *const (),
			call_proc_by_id_hook_trampoline as *const (),
		)
		.unwrap();

		call_hook.enable().unwrap();
		call_proc_by_id_original = std::mem::transmute(call_hook.trampoline());
		std::mem::forget(call_hook);
	}
	Ok(())
}

#[derive(Clone, Copy)]
pub struct ProcHook(pub fn(&Value, &Value, &mut Vec<Value>) -> DMResult);

use std::fmt;

impl fmt::Debug for ProcHook {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "function pointers with more than one ref argument don't actually work with debug, funny!")
	}
}

static mut SHOULD_GENERATE_HOOK_MAP: bool = false;

static mut HOOKED_PROC_PAIRS: Option<Vec<(raw_types::procs::ProcId,ProcHook)>> = None;

// only the byond thread touches this so explicit thread local overhead is unnecessary
static mut PROC_HOOKS: Option<NoKeyBoomHashMap<raw_types::procs::ProcId, ProcHook>> = None;

fn hook_by_id(id: raw_types::procs::ProcId, hook: ProcHook) -> Result<(), HookFailure> {
	unsafe {
		if HOOKED_PROC_PAIRS.is_none() {
			HOOKED_PROC_PAIRS = Some(Vec::new());
		}
		for &(extant_id,_) in HOOKED_PROC_PAIRS.as_ref().unwrap().iter() {
			if extant_id == id {
				return Err(HookFailure::AlreadyHooked);
			}
		}
		HOOKED_PROC_PAIRS.as_mut().unwrap().push((id,hook));
		if SHOULD_GENERATE_HOOK_MAP {
			generate_hook_map();
		}
		Ok(())
	}
}

pub fn clear_hooks() {
	unsafe {
		PROC_HOOKS = None;
		HOOKED_PROC_PAIRS = None;
		SHOULD_GENERATE_HOOK_MAP = false;
	}
}

pub fn hook<S: Into<String>>(name: S, hook: ProcHook) -> Result<(), HookFailure> {
	match super::proc::get_proc(name) {
		Some(p) => hook_by_id(p.id, hook),
		None => Err(HookFailure::ProcNotFound),
	}
}

pub fn generate_hook_map() {
	unsafe {
		let keys = HOOKED_PROC_PAIRS.as_ref().unwrap().iter().copied().map(|(k,_)| k).collect::<Vec<_>>();
		let hash = Mphf::new_parallel(100.0, &keys, None);
		HOOKED_PROC_PAIRS.as_mut().unwrap().sort_by_cached_key(|k| hash.hash(&k.0));
		PROC_HOOKS = Some(NoKeyBoomHashMap::new_with_mphf(
			hash,
			HOOKED_PROC_PAIRS.as_ref().unwrap().iter().copied().map(|(_,v)| v).collect(),
		));
		SHOULD_GENERATE_HOOK_MAP = true;
	}
}

impl Proc {
	pub fn hook(&self, func: ProcHook) -> Result<(), HookFailure> {
		hook_by_id(self.id, func)
	}
}

#[no_mangle]
extern "C" fn on_runtime(error: *const c_char) {
	let str = unsafe { CStr::from_ptr(error) }.to_string_lossy();

	for func in inventory::iter::<RuntimeHook> {
		func.0(&str);
	}
}

#[no_mangle]
extern "C" fn call_proc_by_id_hook(
	ret: *mut raw_types::values::Value,
	usr_raw: raw_types::values::Value,
	_proc_type: u32,
	proc_id: raw_types::procs::ProcId,
	_unknown1: u32,
	src_raw: raw_types::values::Value,
	args_ptr: *mut raw_types::values::Value,
	num_args: usize,
	_unknown2: u32,
	_unknown3: u32,
) -> u8 {
	match unsafe { PROC_HOOKS.as_ref().unwrap() }.get(&proc_id) {
		Some(hook) => {
			let src;
			let usr;
			let mut args: Vec<Value>;

			unsafe {
				src = Value::from_raw(src_raw);
				usr = Value::from_raw(usr_raw);

				// Taking ownership of args here
				args = std::slice::from_raw_parts(args_ptr, num_args)
					.iter()
					.map(|v| Value::from_raw_owned(*v))
					.collect();
			}

			let result = hook.0(&src, &usr, &mut args);

			match result {
				Ok(r) => {
					let result_raw = (&r).raw;
					// Stealing our reference out of the Value
					std::mem::forget(r);
					unsafe {
						*ret = result_raw;
					}
				}
				Err(e) => {
					// TODO: Some info about the hook would be useful (as the hook is never part of byond's stack, the runtime won't show it.)
					Proc::find("/proc/auxtools_stack_trace")
						.unwrap()
						.call(&[&Value::from_string(e.message.as_str()).unwrap()])
						.unwrap();
					unsafe {
						*ret = Value::null().raw;
					}
				}
			}
			1
		}
		None => 0,
	}
}
