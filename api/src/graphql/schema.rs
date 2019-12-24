use juniper::{graphql_value, Context as JuniperContext, FieldResult, ID};
use mongodb::{coll::Collection, db::ThreadedDatabase, oid::ObjectId, Bson};

use crate::{
	db::PrimaryDb,
	models::{Address, CollectionMethod, Order, PostDeliveryOption, Postage, User},
};

const SCARVE_PRICE : f64 = 30.0;

pub struct Context {
	pub connection : PrimaryDb,
}

impl Context {
	pub fn orders_handel(&self) -> Collection { self.connection.collection("orders") }
}

impl JuniperContext for Context {}

#[juniper::object(description = "Contact Details of the person making the purchase")]
impl User {
	/// Contact name
	fn name(&self) -> &str { &self.name }

	/// Contact email
	fn email(&self) -> &str { &self.email }
}

#[juniper::object(
	description = "The root order. This holds all details on an order including contact, address and postage information"
)]
impl Order {
	fn id(&self) -> &ID { &self.id }

	/// Contact details
	fn user(&self) -> User { self.user.clone() }

	/// delivery address
	fn address(&self) -> Option<Address> { self.address.clone() }

	/// postage details
	fn postage(&self) -> Option<Postage> { self.postage.clone() }

	/// quantity of scarves to be delivered
	fn quantity(&self) -> i32 { self.quantity }

	/// is the item Picked up or delivered
	fn method(&self) -> CollectionMethod { self.method }
}

#[juniper::object(description = "Delivery Address")]
impl Address {
	fn apartment(&self) -> Option<String> { self.apartment.clone() }

	fn street(&self) -> &str { &self.street }

	fn town(&self) -> &str { &self.town }

	fn state(&self) -> &str { &self.state }

	fn post_code(&self) -> i32 { self.post_code }
}

#[juniper::object(description = "The type of postage the user has selected, and the price")]
impl Postage {
	fn typ(&self) -> &str { &self.typ }

	fn price(&self) -> f64 { self.price }
}

#[juniper::object(description = "A post delivery option from Australia Post")]
impl PostDeliveryOption {
	/// The name of the delivery option
	fn name(&self) -> &str { &self.name }

	/// The price of the deliver option as stated by the API
	fn price(&self) -> f64 { self.price }
}

pub struct QueryRoot;
#[juniper::object(
    Context = Context,
)]
impl QueryRoot {
	/// All orders
	fn orders(context : &Context) -> Vec<Order> {
		let orders = context.orders_handel();
		orders
			.find(None, None)
			.unwrap()
			.into_iter()
			.filter_map(|item| match item {
				Ok(item) => Some(Order::from_doc(item)),
				Err(_) => None,
			})
			.collect()
	}

	/// For an order, calculate the price to post the items to the user
	fn calculatePostage(context : &Context, id : String) -> FieldResult<Vec<PostDeliveryOption>> {
		let orders = context.orders_handel();
		let order = match orders.find_one(
			Some(doc! {
				"_id" => match mongodb::oid::ObjectId::with_string(&id) {
					Ok(oid) => oid,
					Err(_) => return Err(juniper::FieldError::new(
					"UID is not valid",
					graphql_value!({
						"type": "INVALID_UID"
					}),
				))

				}
			}),
			None,
		) {
			Ok(Some(o)) => Order::from_doc(o),
			_ => {
				return Err(juniper::FieldError::new(
					"Internal Error decoding Document from database",
					graphql_value!({
						"type": "DATABASE_ERROR"
					}),
				))
			},
		};

		let postcode = match order.address {
			Some(addr) => addr.post_code,
			None => {
				return Err(juniper::FieldError::new(
					"This order does not have an address defined. This is likely because the Pickup option was selected",
					graphql_value!({
						"type": "NO_ADDRESS"
					}),
				))
			},
		};

		match PostDeliveryOption::get(order.quantity as u32, postcode as u32) {
			Ok(opts) => Ok(opts),
			Err(_) => Err(juniper::FieldError::new(
				"Quantity must be greater than 0",
				graphql_value!({
					"type": "INVALID_QUANTITY"
				}),
			)),
		}
	}

	/// Return the price of the order, excluding postage
	fn orderPrice(context : &Context, id : String) -> FieldResult<f64> {
		let orders = context.orders_handel();
		let order = match orders.find_one(
			Some(doc! {
				"_id" => match mongodb::oid::ObjectId::with_string(&id) {
					Ok(oid) => oid,
					Err(_) => return Err(juniper::FieldError::new(
					"UID is not valid",
					graphql_value!({
						"type": "NO_WHATEVER"
					}),
				))

				}
			}),
			None,
		) {
			Ok(Some(o)) => Order::from_doc(o),
			_ => {
				return Err(juniper::FieldError::new(
					"Internal Error decoding Document from database",
					graphql_value!({
						"type": "NO_WHATEVER"
					}),
				))
			},
		};

		Ok(order.quantity as f64 * SCARVE_PRICE)
	}
}

pub struct MutationRoot;

#[juniper::object(
    Context = Context
)]
impl MutationRoot {
	/// Take in the details of a user, how they would like to receive their
	/// order and possibly their address.
	fn newOrder(
		context : &Context,
		name : String,
		quantity : i32,
		email : String,
		address_apt : Option<String>,
		address_street : Option<String>,
		address_town : Option<String>,
		address_state : Option<String>,
		address_post_code : Option<i32>,
		delivery_method : CollectionMethod,
	) -> FieldResult<Option<Order>> {
		if quantity < 1 {
			return Err(juniper::FieldError::new(
				"Quantity must be greater than 0",
				graphql_value!({
					"type": "NO_WHATEVER"
				}),
			));
		};
		let orders = context.orders_handel();

		let result = orders
			.insert_one(
				doc! {
				"quantity" => quantity,
				"user" => {
					"name" => name,
					"email" => email,
				},
				"method" => match delivery_method {
					CollectionMethod::Pickup => 1,
					CollectionMethod::Post => 0,
				} as i32,
				},
				None,
			)
			.unwrap();

		match delivery_method {
			CollectionMethod::Post => {
				orders
					.update_one(
						doc! {"_id" => result.inserted_id.clone().unwrap() },
						doc! {
							"$set" => {
								"address" => {
									"apartment" => match address_apt {Some(a) => a, None => "".to_string()},
									"street" => match address_street{Some(a) => a, None => "".to_string()},
									"town" => match address_town{Some(a) => a, None => "".to_string()},
									"state" => match address_state{Some(a) => a, None => "".to_string()},
									"post_code" => match address_post_code{Some(a) => a, None => 0},
								}
							}
						},
						None,
					)
					.unwrap();
			},
			_ => {},
		};

		let id = result
			.inserted_id
			.unwrap_or(Bson::ObjectId(ObjectId::new().unwrap()));

		match orders.find_one(Some(doc! {"_id" => id}), None) {
			Ok(Some(order)) => Ok(Some(Order::from_doc(order))),
			Ok(None) => Ok(None),
			_ => Err(juniper::FieldError::new(
				"Whatever does not exist",
				graphql_value!({
					"type": "NO_WHATEVER"
				}),
			)),
		}
	}

	/// Set the requested postage method from the user. Once this is done the
	/// order is practically finalized and just needs to be paid for.
	fn setPostage(
		context : &Context,
		id : String,
		typ : String,
		price : f64,
	) -> FieldResult<Order> {
		let orders = context.orders_handel();

		orders
			.update_one(
				doc! {
				"_id" => match mongodb::oid::ObjectId::with_string(&id) {
					Ok(oid) => oid,
					Err(_) => return Err(juniper::FieldError::new(
					"UID is not valid",
					graphql_value!({
						"type": "NO_WHATEVER"
					}),
				))
				}},
				doc! {
					"$set" => {
						"postage" => {
							"type" => typ,
							"price" => price,
						}
					}
				},
				None,
			)
			.unwrap();

		match orders.find_one(
			Some(doc! {
			"_id" => match mongodb::oid::ObjectId::with_string(&id) {
				Ok(oid) => oid,
				Err(_) => return Err(juniper::FieldError::new(
				"UID is not valid",
				graphql_value!({
					"type": "NO_WHATEVER"
				}),
			))
			}}),
			None,
		) {
			Ok(Some(order)) => Ok(Order::from_doc(order)),
			_ => {
				return Err(juniper::FieldError::new(
					"UID is not valid",
					graphql_value!({
						"type": "NO_WHATEVER"
					}),
				))
			},
		}
	}
}
