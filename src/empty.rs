#![no_std]

use multiversx_sc::types::heap::Vec;

multiversx_sc::imports!();
multiversx_sc::derive_imports!();


#[multiversx_sc::contract]
pub trait EsdtContract {
    #[init]
    fn init(&self) {
    }

    #[endpoint]
    #[payable("*")]
    fn deposit(&self) {
        let (token, _, payment) = self.call_value().egld_or_single_esdt().into_tuple();

        let caller = self.blockchain().get_caller();
        self.storage(&caller, &token).update(|deposit| *deposit += payment);
    }

    #[endpoint]
    fn withdraw(&self, amount: &BigUint, token: &EgldOrEsdtTokenIdentifier) {
        let caller = self.blockchain().get_caller();
        let deposit = self.storage(&caller, &token).get();

        if deposit - amount >= 0u32 {
            self.storage(&caller, &token).update(|deposit_val| *deposit_val -= amount);
            self.send().direct(&caller, &token, 0, &amount);
        }
     }

     #[endpoint]
     fn safe_batch_transfer_from(
        &self,
        from: &ManagedAddress,
        to: &ManagedAddress, 
        ids: Vec<EgldOrEsdtTokenIdentifier>, 
        values: Vec<BigUint>
    ) {
        let caller = self.blockchain().get_caller();
        require!(&caller == from || (self.approval(&to, &from).get() && caller == *to), "only the caller can move his funds");
        require!(ids.len() == values.len(), "different lengths of arrays");
    
        for i in 0..ids.len() {
            let token_id = ids[i].clone();
            let value = values[i].clone();
            let deposit = self.storage(&from, &token_id).get();
            require!(deposit >= value, "cannot transfer a higher value than what you own");
            self.storage(&from, &token_id).update(|deposit_val| *deposit_val -= value.clone());
            self.storage(&to, &token_id).update(|deposit_val| *deposit_val += value.clone());
        }
    }

    #[endpoint]
    fn balance_of_batch(
        &self,
        owners: Vec<ManagedAddress>,
        ids: Vec<EgldOrEsdtTokenIdentifier>
    ) -> Vec<BigUint> {
        require!(owners.len() == ids.len() , "different lengths of arrays");

        let mut balances = Vec::new();
        for i in 0..owners.len() {
            let owner = owners[i].clone();
            let id = ids[i].clone();
            //in case that a value was never deposited for a specific token this should return 0.
            let amount = self.storage(&owner, &id).get();
            balances.push(amount);
        }

        return balances;
    }

    #[endpoint]
    fn set_approval_for_all(
        &self,
        operator : &ManagedAddress,
        approved : bool
    ) {
        let caller = self.blockchain().get_caller();
        if approved {
            self.approval(&operator, &caller).set(approved);
        }
        else{
            //this is for protecting the sc from storing irelevant data. 
            //After all, we are only interested in knowing what user can move the funds of a particular user
            //Also, protects from malicious actors calling this method with random operator and approved set on false
            //which would increase the smart contract space
            self.approval(&operator, &caller).clear();
        }
    }
    
    fn on_erc1155_batch_received(
        &self,
        _operator: &ManagedAddress,
        _from: &ManagedAddress,
        _ids: &ManagedVec<EgldOrEsdtTokenIdentifier>,
        _values: &ManagedVec<BigUint>,
        _data: &Vec<u8>,
    ) -> bool {
        return true;
    }

    #[view(getDeposit)]
    #[storage_mapper("updatestorage")]
    fn storage(&self, donor: &ManagedAddress, token_identifier: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<BigUint>;

    #[view(getApproval)]
    #[storage_mapper("approval")]
    fn approval(&self, operator: &ManagedAddress, from: &ManagedAddress) -> SingleValueMapper<bool>;

}