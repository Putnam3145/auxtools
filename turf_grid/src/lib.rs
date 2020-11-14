use dm::*;

use std::sync::atomic::{AtomicI32,Ordering};

pub struct TurfGrid {}
static MAX_X : AtomicI32 = AtomicI32::new(255);
static MAX_Y : AtomicI32 = AtomicI32::new(255);
static MAX_Z : AtomicI32 = AtomicI32::new(1);

impl TurfGrid {
	pub fn refresh_grid(ctx: &DMContext) -> DMResult {
		let world = ctx.get_world();
		let new_x = world.get_number("maxx")? as i32;
		let new_y = world.get_number("maxy")? as i32;
		let new_z = world.get_number("maxz")? as i32;
		MAX_X.store(new_x,Ordering::Relaxed);
		MAX_Y.store(new_y,Ordering::Relaxed);
		MAX_Z.store(new_z,Ordering::Relaxed);
		Ok(Value::from(true))
	}
	pub fn max_x() -> i32 {
		MAX_X.load(Ordering::Relaxed)
	}
	pub fn max_y() -> i32 {
		MAX_Y.load(Ordering::Relaxed)
	}
	pub fn max_z() -> i32 {
		MAX_Z.load(Ordering::Relaxed)
	}
	pub fn max_id() -> i32 {
		TurfGrid::max_x() * TurfGrid::max_y() * TurfGrid::max_z()
	}
	pub fn to_id(x: i32, y: i32, z: i32) -> Result<u32, Runtime> {
		let cur_max_x = TurfGrid::max_x();
		let cur_max_y = TurfGrid::max_y();
		let x = x - 1;
		let y = y - 1;
		let z = z - 1;
		if (0..cur_max_x).contains(&x)
			&& (0..cur_max_y).contains(&y)
			&& (0..TurfGrid::max_z()).contains(&z)
		{
			Ok((x + y * cur_max_x + z * cur_max_x * cur_max_y) as u32)
		} else {
			Err(runtime!("Attempted to get out-of-range tile."))
		}
	}
	pub fn turf_by_id(id: u32) -> Value {
		let tag = raw_types::values::ValueTag::Turf;
		let data = raw_types::values::ValueData { id: id };
		unsafe { Value::new(tag, data) }
	}
	pub fn turf_ref(x: i32, y: i32, z: i32) -> DMResult {
		Ok(TurfGrid::turf_by_id(TurfGrid::to_id(x, y, z)?)) // TODO: implement Value::turf
	}
}

#[hook("/world/proc/refresh_atmos_grid")]
fn _refresh_atmos_grid_hook() {
	TurfGrid::refresh_grid(ctx)
}

#[cfg(test)]
mod tests {
	#[test]
	fn it_works() {
		assert_eq!(2 + 2, 4);
	}
}
