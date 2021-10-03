use std::collections::HashMap;

use once_cell::sync::Lazy;
use secret_service::{Collection, EncryptionType, Error, SecretService};

static SECRET_SERVICE: Lazy<SecretService<'static>> =
	Lazy::new(|| SecretService::new(EncryptionType::Dh).expect("failed connecting to secret service, is it running?"));

#[derive(Debug, Clone, Copy)]
pub enum ServiceProvider {
	MicrosoftO365,
	SchulmanagerJwt
}
impl ServiceProvider {
	pub fn get_id(&self) -> &'static str {
		match self {
			Self::MicrosoftO365 => "o365",
			Self::SchulmanagerJwt => "sm"
		}
	}

	pub fn get_name(&self) -> &'static str {
		match self {
			Self::MicrosoftO365 => "Microsoft Office 365 Password",
			Self::SchulmanagerJwt => "Schulmanager JSON Web Token"
		}
	}
}

fn ss_attrs(service: ServiceProvider, account: &str) -> HashMap<&'static str, &str> {
	let mut attrs = HashMap::new();
	attrs.insert("app_id", crate::config::APP_ID);
	attrs.insert("service", service.get_id());
	attrs.insert("account", &account);

	attrs
}

pub struct Secrets;
impl Secrets {
	pub fn get_default_collection<'a>() -> Result<Collection<'a>, Error> {
		let collection = match SECRET_SERVICE.get_default_collection() {
			Err(Error::NoResult) => SECRET_SERVICE.create_collection("default", "default"),
			e => e
		}?;

		Ok(collection)
	}

	pub fn ensure_unlocked() -> Result<(), Error> {
		let collection = Self::get_default_collection()?;
		collection.unlock()?;

		Ok(())
	}

	pub fn get_secret(service: ServiceProvider, account: &str) -> Result<Option<String>, Error> {
		let col = Self::get_default_collection()?;

		let items = col.search_items(ss_attrs(service, account))?;
		Ok(items
			.get(0)
			.map(|e| Some(String::from_utf8(e.get_secret().ok()?).unwrap()))
			.flatten())
	}

	pub fn set_secret(service: ServiceProvider, account: &str, secret: &str) -> Result<(), Error> {
		let col = Self::get_default_collection()?;

		col.create_item(
			&format!("Magnesium: {}", service.get_name()),
			ss_attrs(service, account),
			secret.as_bytes(),
			true,
			"text/plain"
		)?;

		Ok(())
	}

	pub fn remove_secret(service: ServiceProvider, account: &str) -> Result<(), Error> {
		let col = Self::get_default_collection()?;

		let items = col.search_items(ss_attrs(service, account))?;

		match items.get(0) {
			Some(i) => i.delete(),
			None => Err(Error::NoResult)
		}
	}
}
