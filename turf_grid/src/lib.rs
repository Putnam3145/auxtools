use dm::*;

use std::cell::RefCell;

pub struct TurfGrid {}

impl TurfGrid {
	thread_local! {
		static MAX_X : RefCell<i32> = RefCell::new(255);
		static MAX_Y : RefCell<i32> = RefCell::new(255);
		static MAX_Z : RefCell<i32> = RefCell::new(1);
	}
	pub fn refresh_grid(ctx: &DMContext) -> Result<Value, Runtime> {
		let world = ctx.get_world();
		let new_x = world.get_number("maxx")? as i32;
		let new_y = world.get_number("maxy")? as i32;
		let new_z = world.get_number("maxz")? as i32;
		TurfGrid::MAX_X.with(|x| *x.borrow_mut() = new_x);
		TurfGrid::MAX_Y.with(|y| *y.borrow_mut() = new_y);
		TurfGrid::MAX_Z.with(|z| *z.borrow_mut() = new_z);
		Ok(Value::from(true))
	}
	pub fn max_x() -> i32 {
		TurfGrid::MAX_X.with(|x| -> i32 { *x.borrow() })
	}
	pub fn max_y() -> i32 {
		TurfGrid::MAX_Y.with(|y| -> i32 { *y.borrow() })
	}
	pub fn max_z() -> i32 {
		TurfGrid::MAX_Y.with(|z| -> i32 { *z.borrow() })
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
	pub fn turf_ref(x: i32, y: i32, z: i32) -> Result<Value, Runtime> {
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
