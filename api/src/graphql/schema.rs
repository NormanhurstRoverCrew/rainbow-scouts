use juniper::{Context as JuniperContext, ID};
use mongodb::{
	coll::{options::WriteModel, Collection},
	db::ThreadedDatabase,
	oid::ObjectId,
	Bson,
};

use crate::{
	db::PrimaryDb,
	models::{Purchase, Todo, User},
};

pub struct Context {
	pub connection : PrimaryDb,
}

impl Context {
	pub fn todos_handle(&self) -> Collection { self.connection.collection("todos") }
	pub fn purchases_handel(&self) -> Collection { self.connection.collection("purchases") }
}

impl JuniperContext for Context {}

#[juniper::object(description = "A todo item that can be marked as completed")]
impl Todo {
	/// The unique id of the todo item
	fn id(&self) -> &ID { &self.id }

	/// The user-editable title
	fn title(&self) -> &str { &self.title }

	/// Determines whether the user has completed the item or not
	fn completed(&self) -> bool { self.completed }
}

#[juniper::object(description = "A Purchase holds the payment information for multiple tickets")]
impl User {
	/// The unique id of the todo item
	fn id(&self) -> &ID { &self.id }

	/// The user-editable title
	fn name(&self) -> &str { &self.name }

	/// Determines whether the user has completed the item or not
	fn email(&self) -> Option<String> { self.email.clone() }

	/// Eh
	fn mobile(&self) -> Option<String> { self.mobile.clone() }

	/// Eh
	fn crew(&self) -> Option<String> { self.crew.clone() }
}

#[juniper::object(description = "A Purchase holds the payment information for multiple tickets")]
impl Purchase {
	/// The unique id of the todo item
	fn id(&self) -> &ID { &self.id }

	/// The user-editable title
	fn users(&self) -> Vec<User> { self.users.clone() }
}

pub struct QueryRoot;

#[juniper::object(
    Context = Context,
)]
impl QueryRoot {
	/// An array of every purchase
	fn purchases(context : &Context) -> Vec<Purchase> {
		let purchases = context.purchases_handel();
		purchases
			.find(None, None)
			.unwrap()
			.into_iter()
			.filter_map(|item| match item {
				Ok(item) => Some(Purchase::from_doc(item)),
				Err(_) => None,
			})
			.collect()
	}

	/// An array of every user that has a ticket
	fn users(context : &Context) -> Vec<User> {
		let purchases = context.purchases_handel();
		purchases
			.find(None, None)
			.unwrap()
			.into_iter()
			.filter_map(|item| match item {
				Ok(item) => Some(Purchase::from_doc(item)),
				Err(_) => None,
			})
			.flat_map(|item| {
				item.users
					.iter()
					.map(|user| user.to_owned())
					.collect::<Vec<User>>()
			})
			.collect()
	}

	/// Returns an array of all the crews that have a participant
	fn crews(context : &Context) -> Vec<String> {
		let purchases = context.purchases_handel();
		purchases
			.find(None, None)
			.unwrap()
			.into_iter()
			.filter_map(|item| match item {
				Ok(item) => Some(Purchase::from_doc(item)),
				Err(_) => None,
			})
			.flat_map(|item| {
				item.users
					.iter()
					.filter_map(|user| user.crew.clone())
					.collect::<Vec<String>>()
			})
			.collect()
	}
}

pub struct MutationRoot;

#[juniper::object(
    Context = Context
)]
impl MutationRoot {
	fn newPurchase(context : &Context, name : String) -> Option<Purchase> {
		let purchases = context.purchases_handel();
		let result = purchases
			.insert_one(
				doc! {
					"users" => [{
						"name" => name
					}]
				},
				None,
			)
			.unwrap();

		let id = result
			.inserted_id
			.unwrap_or(Bson::ObjectId(ObjectId::new().unwrap()));

		match purchases.find_one(Some(doc! {"_id" => id}), None) {
			Ok(Some(purchase)) => Some(Purchase::from_doc(purchase)),
			_ => None,
		}
	}
}
