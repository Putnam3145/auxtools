#[macro_use]
extern crate lazy_static;

pub mod gas;

pub mod atmos_grid;

use dm::*;

use atmos_grid::*;

use gas::*;

use gas::constants::*;

#[hook("/datum/gas_mixture/proc/__gasmixture_register")]
fn _register_gasmixture_hook() {
	gas::GasMixtures::register_gasmix(src)
}

#[hook("/datum/gas_mixture/proc/__gasmixture_unregister")]
fn _unregister_gasmixture_hook() {
	gas::GasMixtures::unregister_gasmix(src)
}

#[hook("/datum/gas_mixture/proc/heat_capacity")]
fn _heat_cap_hook() {
	with_mix(src, |mix| Ok(Value::from(mix.heat_capacity())))
}

#[hook("/datum/gas_mixture/proc/set_min_heat_capacity")]
fn _min_heat_cap_hook() {
	if args.is_empty() {
		Err(runtime!(
			"attempted to set min heat capacity with no argument"
		))
	} else {
		with_mix_mut(src, |mix| {
			mix.min_heat_capacity = args[0].as_number().unwrap_or(0.0);
			Ok(Value::null())
		})
	}
}

#[hook("/datum/gas_mixture/proc/total_moles")]
fn _total_moles_hook() {
	with_mix(src, |mix| Ok(Value::from(mix.total_moles())))
}

#[hook("/datum/gas_mixture/proc/return_pressure")]
fn _return_pressure_hook() {
	with_mix(src, |mix| Ok(Value::from(mix.return_pressure())))
}

#[hook("/datum/gas_mixture/proc/return_temperature")]
fn _return_temperature_hook() {
	with_mix(src, |mix| Ok(Value::from(mix.get_temperature())))
}

#[hook("/datum/gas_mixture/proc/return_volume")]
fn _return_volume_hook() {
	with_mix(src, |mix| Ok(Value::from(mix.volume)))
}

#[hook("/datum/gas_mixture/proc/thermal_energy")]
fn _thermal_energy_hook() {
	with_mix(src, |mix| Ok(Value::from(mix.thermal_energy())))
}

#[hook("/datum/gas_mixture/proc/merge")]
fn _merge_hook() {
	if args.is_empty() {
		Err(runtime!("Tried merging nothing into a gas mixture"))
	} else {
		with_mixes_mut(src, &args[0], |src_mix, giver_mix| {
			src_mix.merge(giver_mix);
			Ok(Value::null())
		})
	}
}

#[hook("/datum/gas_mixture/proc/__remove_ratio")]
fn _remove_ratio_hook() {
	if args.len() < 2 {
		Err(runtime!("remove_ratio called with fewer than 2 arguments"))
	} else {
		with_mixes_mut(src, &args[0], |src_mix, into_mix| {
			into_mix.copy_from_mutable(&src_mix.remove_ratio(args[1].as_number().unwrap_or(0.0)));
			Ok(Value::null())
		})
	}
}

#[hook("/datum/gas_mixture/proc/__remove")]
fn _remove_hook() {
	if args.len() < 2 {
		Err(runtime!("remove called with fewer than 2 arguments"))
	} else {
		with_mixes_mut(src, &args[0], |src_mix, into_mix| {
			into_mix.copy_from_mutable(&src_mix.remove(args[1].as_number().unwrap_or(0.0)));
			Ok(Value::null())
		})
	}
}

#[hook("/datum/gas_mixture/proc/copy_from")]
fn _copy_from_hook() {
	if args.is_empty() {
		Err(runtime!("Tried copying a gas mix from nothing"))
	} else {
		with_mixes_mut(src, &args[0], |src_mix, giver_mix| {
			src_mix.copy_from_mutable(giver_mix);
			Ok(Value::null())
		})
	}
}

#[hook("/datum/gas_mixture/proc/temperature_share")]
fn _temperature_share_hook() {
	let arg_num = args.len();
	match arg_num {
		2 => with_mixes_mut(src, &args[0], |src_mix, share_mix| {
			Ok(Value::from(src_mix.temperature_share(
				share_mix,
				args[1].as_number().unwrap_or(0.0),
			)))
		}),
		4 => with_mix_mut(src, |mix| {
			Ok(Value::from(mix.temperature_share_non_gas(
				args[0].as_number().unwrap_or(0.0),
				args[1].as_number().unwrap_or(0.0),
				args[2].as_number().unwrap_or(0.0),
			)))
		}),
		_ => Err(runtime!("Invalid args for temperature_share")),
	}
}

#[hook("/datum/gas_mixture/proc/get_gases")]
fn _get_gases_hook() {
	with_mix(src, |mix| {
		let gases_list: List = List::new();
		for gas in mix.get_gases() {
			gases_list.append(&gas_id_to_type(gas as usize)?);
		}
		Ok(Value::from(gases_list))
	})
}

#[hook("/datum/gas_mixture/proc/set_temperature")]
fn _set_temperature_hook() {
	let v = if args.is_empty() {
		0.0
	} else {
		args[0].as_number().unwrap_or(0.0)
	};
	if !v.is_finite() {
		Err(runtime!(
			"Attempted to set a temperature to a number that is NaN or infinite."
		))
	} else {
		with_mix_mut(src, |mix| {
			mix.set_temperature(v.max(2.7));
			Ok(Value::null())
		})
	}
}

#[hook("/datum/gas_mixture/proc/set_volume")]
fn _set_volume_hook() {
	if args.is_empty() {
		Err(runtime!("Attempted to set volume to nothing."))
	} else {
		with_mix_mut(src, |mix| {
			mix.volume = args[0].as_number()?;
			Ok(Value::null())
		})
	}
}

#[hook("/datum/gas_mixture/proc/get_moles")]
fn _get_moles_hook() {
	if args.is_empty() {
		Err(runtime!("Incorrect arg len for get_moles (0)."))
	} else {
		with_mix(src, |mix| {
			Ok(Value::from(mix.get_moles(gas_id_from_type(&args[0])?)))
		})
	}
}

#[hook("/datum/gas_mixture/proc/set_moles")]
fn _set_moles_hook() {
	if args.len() < 2 {
		return Err(runtime!("Incorrect arg len for set_moles (less than 2)."));
	}
	let vf = args[1].as_number().unwrap_or(0.0);
	if !vf.is_finite() {
		return Err(runtime!("Attempted to set moles to NaN or infinity."));
	}
	if vf < 0.0 {
		return Err(runtime!("Attempted to set moles to a negative number."));
	}
	with_mix_mut(src, |mix| {
		mix.set_moles(gas_id_from_type(&args[0])?, vf);
		Ok(Value::null())
	})
}

#[hook("/datum/gas_mixture/proc/scrub_into")]
fn _scrub_into_hook() {
	if args.len() < 2 {
		Err(runtime!("Incorrect arg len for scrub_into (less than 2)."))
	} else {
		with_mixes_mut(src, &args[0], |src_gas, dest_gas| {
			let mixes_to_scrub = args[1].as_list().unwrap();
			let mut buffer = gas::gas_mixture::GasMixture::from_vol(gas::constants::CELL_VOLUME);
			buffer.set_temperature(src_gas.get_temperature());
			for idx in 1..mixes_to_scrub.len() + 1 {
				let res = gas_id_from_type(&mixes_to_scrub.get(idx).unwrap());
				if res.is_ok() {
					// it's allowed to continue after failure here
					let idx = res.unwrap();
					buffer.set_moles(idx, buffer.get_moles(idx) + src_gas.get_moles(idx));
					src_gas.set_moles(idx, 0.0);
				}
			}
			dest_gas.merge(&buffer);
			Ok(args[0].clone())
		})
	}
}

#[hook("/datum/gas_mixture/proc/mark_immutable")]
fn _mark_immutable_hook() {
	with_mix_mut(src, |mix| {
		mix.mark_immutable();
		Ok(Value::null())
	})
}

#[hook("/datum/gas_mixture/proc/clear")]
fn _clear_hook() {
	with_mix_mut(src, |mix| {
		mix.clear();
		Ok(Value::null())
	})
}

#[hook("/datum/gas_mixture/proc/compare")]
fn _compare_hook() {
	if args.is_empty() {
		Err(runtime!("Tried comparing a gas mix to nothing"))
	} else {
		with_mixes(src, &args[0], |gas_one, gas_two| {
			let res = gas_one.compare(gas_two);
			match res {
				-1 => Ok(Value::from_string("temp")),
				-2 => Ok(Value::from_string("")),
				_ => gas_id_to_type(res as usize),
			}
		})
	}
}

#[hook("/datum/gas_mixture/proc/multiply")]
fn _multiply_hook() {
	with_mix_mut(src, |mix| {
		mix.multiply(if args.is_empty() {
			1.0
		} else {
			args[0].as_number().unwrap_or(1.0)
		});
		Ok(Value::null())
	})
}

#[hook("/datum/gas_mixture/proc/react")]
fn _react_hook() {
	let mut ret: i32 = 0;
	let n = Value::null();
	let holder = args.first().unwrap_or(&n);
	let mut reactions: Vec<&gas::reaction::Reaction> = Vec::new();
	with_mix(src, |mix| {
		reactions = mix.all_reactable();
		Ok(Value::null())
	})?;
	for reaction in reactions.iter() {
		ret |= reaction.react(src, holder)?.as_number()? as i32;
		if ret & STOP_REACTIONS == STOP_REACTIONS {
			return Ok(Value::from(ret as f32));
		}
	}
	Ok(Value::from(ret as f32))
}

#[hook("/datum/controller/subsystem/air/proc/get_amt_gas_mixes")]
fn _hook_amt_gas_mixes() {
	Ok(Value::from(
		(amt_non_turf_gases() + amt_turf_gases()) as f32,
	))
}

#[hook("/datum/controller/subsystem/air/proc/get_max_gas_mixes")]
fn _hook_max_gas_mixes() {
	Ok(Value::from(
		(tot_non_turf_gases() + amt_turf_gases()) as f32,
	))
}
