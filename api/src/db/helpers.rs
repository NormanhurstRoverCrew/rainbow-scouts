use crate::db::FromDoc;
use mongodb::{coll::Collection, oid::ObjectId};

pub fn all<T : FromDoc>(coll : Collection) -> Vec<T> {
	coll.find(None, None)
		.unwrap()
		.into_iter()
		.filter_map(|item| match item {
			Ok(item) => Some(T::from_doc(item)),
			Err(_) => None,
		})
		.collect()
}

pub fn get<T : FromDoc>(coll : Collection, id : ObjectId) -> Option<T> {
	match coll.find_one(
		Some(doc! {
			"_id" => id,
		}),
		None,
	) {
		Ok(Some(o)) => Some(T::from_doc(o)),
		_ => None,
	}
}
