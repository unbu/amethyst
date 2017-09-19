use bincode;
use ron;
use ecs::Entity;

error_chain! {
    errors {
        BrokenEntity(entity: Entity) {
            description("The enitity seems broken")
            display("The entity {:?} seems broken", entity)
        }
        BrokenResource(type_name: &'static str) {
            description("The resource seems broken")
            display("The resource {:?} seems broken", type_name)
        }
    }
    foreign_links {
        BincodeError(bincode::internal::Error);
        RonError(ron::de::Error);
    }
}
