use std::marker::PhantomData;

use amethyst_assets::AssetStorage;
use shred::Resource;
use specs::{Fetch, FetchMut, System};
use specs::common::Errors;

use sink::AudioSink;
use source::{Source, SourceHandle};

/// Calls a closure if the `AudioSink` is empty.
pub struct DjSystem<F, R> {
    f: F,
    marker: PhantomData<R>,
}

impl<F, R> DjSystem<F, R> {
    /// Creates a new `DjSystem` with the music picker being `f`.
    /// The closure takes a parameter, which needs to be a reference to
    /// a resource type, e.g. `&MusicLibrary`. This resource will be fetched
    /// by the system and passed to the picker.
    pub fn new(f: F) -> Self {
        DjSystem {
            f,
            marker: PhantomData,
        }
    }
}

impl<'a, F, R> System<'a> for DjSystem<F, R>
where
    F: FnMut(&mut R) -> Option<SourceHandle>,
    R: Resource,
{
    type SystemData = (
        Fetch<'a, AssetStorage<Source>>,
        Fetch<'a, Errors>,
        Fetch<'a, AudioSink>,
        FetchMut<'a, R>,
    );

    fn run(&mut self, (storage, errors, sink, mut res): Self::SystemData) {
        #[cfg(feature = "profiler")]
        profile_scope!("dj_system");
        if sink.empty() {
            if let Some(source) = (&mut self.f)(&mut res).and_then(|h| storage.get(&h)) {
                errors.execute(|| sink.append(source));
            }
        }
    }
}
