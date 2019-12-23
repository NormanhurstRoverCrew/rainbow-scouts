use juniper::{graphql_value, Context as JuniperContext, FieldResult, ID};
use mongodb::{coll::Collection, db::ThreadedDatabase, oid::ObjectId, Bson};

use crate::{
	db::PrimaryDb,
	models::{Address, CollectionMethod, Order, User},
};

pub struct Context {
	pub connection : PrimaryDb,
}

impl Context {
	pub fn orders_handel(&self) -> Collection { self.connection.collection("orders") }
}

impl JuniperContext for Context {}

#[juniper::object(description = "A Order holds the payment information for multiple tickets")]
impl User {
	/// The user-editable title
	fn name(&self) -> &str { &self.name }

	/// Determines whether the user has completed the item or not
	fn email(&self) -> &str { &self.email }
}

#[juniper::object(description = "A Order holds the payment information for multiple tickets")]
impl Order {
	/// The unique id of the todo item
	fn id(&self) -> &ID { &self.id }

	/// The user-editable title
	fn user(&self) -> User { self.user.clone() }

	/// The user-editable title
	fn address(&self) -> Option<Address> { self.address.clone() }

	/// The quantity of scarves to be orded
	fn quantity(&self) -> i32 { self.quantity }

	/// The quantity of scarves to be orded
	fn method(&self) -> CollectionMethod { self.method }
}

#[juniper::object(description = "A Order holds the payment information for multiple tickets")]
impl Address {
	/// The user-editable title
	fn apartment(&self) -> Option<String> { self.apartment.clone() }

	/// The user-editable title
	fn street(&self) -> &str { &self.street }

	/// The quantity of scarves to be orded
	fn town(&self) -> &str { &self.town }

	/// The quantity of scarves to be orded
	fn state(&self) -> &str { &self.state }

	/// The quantity of scarves to be orded
	fn post_code(&self) -> i32 { self.post_code }
}

pub struct QueryRoot;
#[juniper::object(
    Context = Context,
)]
impl QueryRoot {
	/// An array of every order
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
}

pub struct MutationRoot;

#[juniper::object(
    Context = Context
)]
impl MutationRoot {
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
}
