use ink::primitives::AccountId;
use xcm::v3::prelude::*;

#[derive(scale::Decode, scale::Encode, Copy, Clone)]
#[cfg_attr(
    feature = "std",
    derive(scale_info::TypeInfo, ink::storage::traits::StorageLayout)
)]
pub struct MultilocationEncoded {
    pub parents: u8,
    pub para_id: Option<u32>,
    pub addr: AccountId,
}

impl MultilocationEncoded {
    pub fn new(parents: u8, para_id: Option<u32>, addr: AccountId) -> Self {
        Self {
            parents,
            para_id,
            addr,
        }
    }

    pub fn get_context(&self) -> InteriorMultiLocation {
        match self.para_id {
            Some(id) => X1(Parachain(id)),
            None => Here,
        }
    }

    pub fn account(&self) -> AccountId {
        self.addr
    }

    pub fn path_to_chain(&self) -> MultiLocation {
        let context = self.get_context();
        MultiLocation::new(self.parents, context)
    }
}

impl TryFrom<MultiLocation> for MultilocationEncoded {
    type Error = ();

    fn try_from(loc: MultiLocation) -> Result<Self, Self::Error> {
        let parents = loc.parent_count();

        let get_account = |jn| match jn {
            AccountId32 { id, .. } => Ok(AccountId::from(id)),
            _ => Err(()),
        };

        let (para_id, acc) = match loc.interior() {
            X1(jn) => (None, get_account(*jn)?),
            X2(Parachain(id), jn) => (Some(*id), get_account(*jn)?),
            _ => Err(())?,
        };

        let instance = Self::new(parents, para_id, acc);

        Ok(instance)
    }
}

impl From<MultilocationEncoded> for MultiLocation {
    fn from(loc: MultilocationEncoded) -> Self {
        let account = AccountId32 {
            network: None, // Is it ok?
            id: *loc.addr.as_ref(),
        };

        let interior = match loc.para_id {
            Some(id) => X2(Parachain(id), account),
            None => X1(account),
        };

        MultiLocation::new(loc.parents, interior)
    }
}

impl From<&MultilocationEncoded> for MultiLocation {
    fn from(value: &MultilocationEncoded) -> Self {
        (*value).into()
    }
}
