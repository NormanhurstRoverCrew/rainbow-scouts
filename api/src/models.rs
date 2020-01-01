use crate::db::FromDoc;
use juniper::{GraphQLEnum, ID};
use mongodb::{oid::ObjectId, Document};
use reqwest::header;
use serde::Deserialize;

#[derive(Clone, Debug)]
pub struct Order {
	pub id :       ID,
	pub quantity : i32,
	pub address :  Option<Address>,
	pub user :     User,
	pub method :   CollectionMethod,
	pub postage :  Option<Postage>,
	pub payment :  Option<Payment>,
}

impl FromDoc for Order {
	fn from_doc(item : Document) -> Self {
		Self {
			id :       Self::doc_get_id(&item),
			quantity : Self::doc_get_quantity(&item),
			user :     Self::doc_get_user(&item),
			address :  Self::doc_get_address(&item),
			method :   Self::doc_get_method(&item),
			postage :  Self::doc_get_postage(&item),
			payment :  Self::doc_get_payment(&item),
		}
	}
}

impl Order {
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

	pub fn doc_get_postage(item : &Document) -> Option<Postage> {
		match item.get_document("postage") {
			Ok(d) => Some(Postage::from_doc(d.to_owned())),
			_ => None,
		}
	}

	pub fn doc_get_method(item : &Document) -> CollectionMethod {
		match item.get_i32("method") {
			Ok(1) => CollectionMethod::Pickup,
			Ok(0) | _ => CollectionMethod::Post,
		}
	}

	pub fn doc_get_payment(item : &Document) -> Option<Payment> {
		match item.get_document("payment") {
			Ok(d) => Some(Payment::from_doc(d.to_owned())),
			_ => None,
		}
	}
}

#[derive(Clone, Debug)]
pub struct Payment {
	pub stripe : Option<PaymentStripe>,
}

impl Payment {
	pub fn from_doc(item : Document) -> Self {
		Self {
			stripe : Self::doc_get_stripe(&item),
		}
	}

	pub fn doc_get_stripe(item : &Document) -> Option<PaymentStripe> {
		match item.get_document("stripe") {
			Ok(d) => Some(PaymentStripe::from_doc(d.to_owned())),
			_ => None,
		}
	}
}

#[derive(Clone, Debug)]
pub struct PaymentStripe {
	pub pi :            String,
	pub client_secret : Option<String>,
}

impl PaymentStripe {
	pub fn from_doc(item : Document) -> Self {
		Self {
			pi :            Self::doc_get_pi(&item),
			client_secret : None,
		}
	}

	pub fn doc_get_pi(item : &Document) -> String {
		String::from(match item.get_str("pi") {
			Ok(d) => d,
			_ => "",
		})
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
		match item.get_i32("post_code") {
			Ok(c) => c,
			_ => 0,
		}
	}
}

#[derive(Clone, Debug)]
pub struct Postage {
	pub code : String,
}

impl Postage {
	pub fn default() -> Self {
		Self {
			code : String::from(""),
		}
	}

	pub fn from_doc(item : Document) -> Self {
		Self {
			code : Self::doc_get_code(&item),
		}
	}

	pub fn doc_get_code(item : &Document) -> String {
		match item.get_str("code") {
			Ok(c) => String::from(c),
			_ => String::from(""),
		}
	}
}

#[derive(GraphQLEnum, Clone, Copy, Debug)]
pub enum CollectionMethod {
	Pickup,
	Post,
}

#[derive(Deserialize, Debug)]
struct PostPricesServiceOptions {
	pub option : Vec<PostPricesService>,
}

#[derive(Deserialize, Debug)]
struct PostPricesService {
	pub code :            String,
	pub name :            String,
	pub max_extra_cover : Option<u32>,
	pub options :         Option<PostPricesServiceOptions>,
	pub price :           Option<String>,
}

#[derive(Deserialize, Debug)]
struct PostPricesServices {
	pub service : Vec<PostPricesService>,
}

#[derive(Deserialize, Debug)]
struct PostPrices {
	pub services : PostPricesServices,
}

pub struct PostDeliveryOptions {
	pub options : Vec<PostDeliveryOption>,
}

pub struct PostDeliveryOption {
	pub name :  String,
	pub code :  String,
	pub price : f64,
}

pub enum PostDeliveryOptionError {
	ApiError,
}

impl PostDeliveryOption {
	pub fn get(quantity : u32, postcode : u32) -> Result<Vec<Self>, PostDeliveryOptionError> {
		let mut headers = header::HeaderMap::new();
		headers.insert(
			header::HeaderName::from_static("auth-key"),
			header::HeaderValue::from_str(&std::env::var("AUSPOST_PAC_API").unwrap()).unwrap(),
		);
		let client = reqwest::blocking::Client::builder()
			.default_headers(headers)
			.build()
			.unwrap();
		let body : PostPrices = match client
			.get("https://digitalapi.auspost.com.au/postage/parcel/domestic/service.json")
			.query(&[
				("from_postcode", "2077"),
				("to_postcode", &postcode.to_string()),
				("length", "22"),
				("width", "16"),
				("height", "7.7"),
				("weight", &(quantity as f64 * 0.1).to_string()),
			])
			.send()
		{
			Ok(response) => response.json().unwrap(),
			Err(_) => return Err(PostDeliveryOptionError::ApiError),
		};

		Ok(body
			.services
			.service
			.iter()
			.map(|serv| PostDeliveryOption::from_api_service(serv))
			.collect())
	}

	fn from_api_service(service : &PostPricesService) -> Self {
		Self {
			name :  service.name.to_owned(),
			price : match &service.price {
				Some(p) => p.parse::<f64>().unwrap(),
				None => 0.0,
			},
			code :  service.code.to_owned(),
		}
	}
}
