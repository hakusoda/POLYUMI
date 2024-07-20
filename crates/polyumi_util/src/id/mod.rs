use std::{
	cmp::Ordering,
	fmt::{ Debug, Display },
	hash::{ Hash, Hasher },
	marker::PhantomData
};
use uuid::Uuid;
use serde::{ Serialize, Serializer, Deserialize, Deserializer };

pub mod marker;

pub struct Id<T> {
	pub value: Uuid,
	phantom: PhantomData<fn(T) -> T>
}

impl<T> Id<T> {
	pub const fn new(value: Uuid) -> Self {
		Self {
			value,
			phantom: PhantomData
		}
	}
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for Id<T> {}

impl<T> Debug for Id<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Debug::fmt(&self.value, f)
	}
}

impl<T> Default for Id<T> {
	fn default() -> Self {
		Self {
			value: Uuid::new_v4(),
			phantom: PhantomData
		}
	}
}

impl<T> Eq for Id<T> {}

impl<T> Hash for Id<T> {
    fn hash<U: Hasher>(&self, state: &mut U) {
        self.value.hash(state)
    }
}

impl<T> Ord for Id<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl<T> PartialOrd for Id<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Display for Id<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self.value, f)
	}
}

impl<T> From<Uuid> for Id<T> {
	fn from(value: Uuid) -> Self {
		Self::new(value)
	}
}

impl<T> Serialize for Id<T> {
	fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
		self.value.serialize(serializer)
	}
}

impl<'de, T> Deserialize<'de> for Id<T> {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		Ok(Self::new(Uuid::deserialize(deserializer)?))
	}
}