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
    fn withdraw(&self, token: &EgldOrEsdtTokenIdentifier) {
        let caller = self.blockchain().get_caller();
        let deposit = self.storage(&caller, &token).get();

        if deposit > 0u32 {
            self.storage(&caller, &token).clear();
            self.send().direct(&caller, &token, 0, &deposit);
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
        //should we treat all transfers as one transaction, meaning to not perform any transfer if one of them can not be performed?
        require!(&self.blockchain().get_caller() == from, "only the caller can move his funds");
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
    

    #[view(getDeposit)]
    #[storage_mapper("updatestorage")]
    fn storage(&self, donor: &ManagedAddress, token_identifier: &EgldOrEsdtTokenIdentifier) -> SingleValueMapper<BigUint>;

}