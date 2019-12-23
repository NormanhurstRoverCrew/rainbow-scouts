use juniper::{GraphQLEnum, ID};
use mongodb::{oid::ObjectId, Document};

pub struct Order {
	pub id :       ID,
	pub quantity : i32,
	pub address :  Option<Address>,
	pub user :     User,
	pub method :   CollectionMethod,
}

impl Order {
	pub fn from_doc(item : Document) -> Self {
		Self {
			id :       Self::doc_get_id(&item),
			quantity : Self::doc_get_quantity(&item),
			user :     Self::doc_get_user(&item),
			address :  Self::doc_get_address(&item),
			method :   Self::doc_get_method(&item),
		}
	}

	pub fn doc_get_id(item : &Document) -> ID {
		ID::from(match item.get_object_id("_id") {
			Ok(oid) => oid.to_string(),
			_ => ObjectId::new().unwrap().to_string(),
		})
	}

	pub fn doc_get_quantity(item : &Document) -> i32 {
		match item.get_i32("quantity") {
			Ok(q) => q as i32,
			_ => 0,
		}
	}

	pub fn doc_get_user(item : &Document) -> User {
		match item.get_document("user") {
			Ok(d) => User::from_doc(d.to_owned()),
			_ => User::default(),
		}
	}

	pub fn doc_get_address(item : &Document) -> Option<Address> {
		match item.get_document("address") {
			Ok(d) => Some(Address::from_doc(d.to_owned())),
			_ => None,
		}
	}

	pub fn doc_get_method(item : &Document) -> CollectionMethod {
		match item.get_i32("method") {
			Ok(1) => CollectionMethod::Pickup,
			Ok(0) | _ => CollectionMethod::Post,
		}
	}
}

#[derive(Clone, Debug)]
pub struct User {
	pub name :  String,
	pub email : String,
}

impl User {
	pub fn default() -> Self {
		Self {
			name :  "".to_string(),
			email : "".to_string(),
		}
	}

	pub fn from_doc(item : Document) -> Self {
		Self {
			name :  Self::doc_get_name(&item),
			email : Self::doc_get_email(&item),
		}
	}

	pub fn doc_get_name(item : &Document) -> String {
		String::from(match item.get_str("name") {
			Ok(t) => t,
			_ => "",
		})
	}

	pub fn doc_get_email(item : &Document) -> String {
		match item.get_str("email") {
			Ok(c) => String::from(c),
			_ => String::from(""),
		}
	}
}

#[derive(Clone, Debug)]
pub struct Address {
	pub apartment : Option<String>,
	pub street :    String,
	pub town :      String,
	pub state :     String,
	pub post_code : i32,
}

impl Address {
	pub fn default() -> Self {
		Self {
			apartment : None,
			street :    "".to_string(),
			town :      "".to_string(),
			state :     "".to_string(),
			post_code : 0,
		}
	}

	pub fn from_doc(item : Document) -> Self {
		Self {
			apartment : Self::doc_get_apartment(&item),
			street :    Self::doc_get_street(&item),
			town :      Self::doc_get_town(&item),
			state :     Self::doc_get_state(&item),
			post_code : Self::doc_get_post_code(&item),
		}
	}

	pub fn doc_get_apartment(item : &Document) -> Option<String> {
		match item.get_str("apartment") {
			Ok(t) => Some(String::from(t)),
			_ => None,
		}
	}

	pub fn doc_get_street(item : &Document) -> String {
		match item.get_str("street") {
			Ok(c) => String::from(c),
			_ => String::from(""),
		}
	}

	pub fn doc_get_town(item : &Document) -> String {
		match item.get_str("town") {
			Ok(c) => String::from(c),
			_ => String::from(""),
		}
	}

	pub fn doc_get_state(item : &Document) -> String {
		match item.get_str("state") {
			Ok(c) => String::from(c),
			_ => String::from(""),
		}
	}

	pub fn doc_get_post_code(item : &Document) -> i32 {
		match item.get_i32("postcode") {
			Ok(c) => c,
			_ => 0,
		}
	}
}

#[derive(GraphQLEnum, Clone, Copy, Debug)]
pub enum CollectionMethod {
	Pickup,
	Post,
}
