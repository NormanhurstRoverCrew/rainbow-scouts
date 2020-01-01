use crate::models::{
	Address, CollectionMethod, Order, Payment, PaymentStripe, PostDeliveryOption, Postage, User,
};
use juniper::ID;

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

	fn payment(&self) -> Option<Payment> { self.payment.clone() }
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
	fn code(&self) -> &str { &self.code }
}

#[juniper::object(description = "A post delivery option from Australia Post")]
impl PostDeliveryOption {
	/// The name of the delivery option
	fn name(&self) -> &str { &self.name }

	/// The price of the deliver option as stated by the API
	fn price(&self) -> f64 { self.price }

	fn code(&self) -> &str { &self.code }
}

#[juniper::object]
#[derive(Clone, Debug)]
impl Payment {
	fn stripe(&self) -> Option<PaymentStripe> { self.stripe.clone() }
}

#[juniper::object]
#[derive(Clone, Debug)]
impl PaymentStripe {
	fn client_secret(&self) -> Option<String> { self.client_secret.clone() }
}
