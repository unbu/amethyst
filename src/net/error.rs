use bincode;
use ecs::Entity;

use net::NetId;
use ron;

error_chain! {
    foreign_links {
        BincodeError(bincode::internal::Error);
        RonError(ron::de::Error);
    }
    errors {
        BrokenEntity(entity: Entity) {
            description("The enitity seems broken")
            display("The entity {:?} seems broken", entity)
        }
        BrokenResource(type_name: &'static str) {
            description("The resource seems broken")
            display("The resource {:?} seems broken", type_name)
        }
        SyncWrongOwner(updater: NetId, id: NetId, owner: NetId) {
            description("Non-owner sync requested")
            display("Node {:?} tried to sync {:?} owned by {:?}", updater, id, owner)
        }
    }
}
