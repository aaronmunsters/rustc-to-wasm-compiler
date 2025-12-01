use crate::configuration::Configuration;

#[derive(Clone, Debug)]
pub struct ConfigurationBuilder<Profile, Debugging, StackSize, Source, Filename> {
    profile: Profile,
    debugging: Debugging,
    stack_size: StackSize,
    source: Source,
    filename: Filename,
}

impl ConfigurationBuilder<(), (), (), (), ()> {
    #[must_use]
    pub fn init() -> Self {
        Self {
            profile: (),
            debugging: (),
            stack_size: (),
            source: (),
            filename: (),
        }
    }
}

impl
    ConfigurationBuilder<
        crate::configuration::Profile,
        crate::configuration::Debugging,
        crate::configuration::StackSize,
        crate::configuration::Source,
        crate::configuration::Filename,
    >
{
    #[must_use]
    pub fn build(self) -> Configuration {
        let Self {
            profile,
            debugging,
            stack_size,
            source,
            filename,
        } = self;

        Configuration {
            profile,
            debugging,
            stack_size,
            source,
            filename,
        }
    }
}

impl<Profile, Debugging, StackSize, Source, Filename>
    ConfigurationBuilder<Profile, Debugging, StackSize, Source, Filename>
{
    pub fn profile(
        self,
        profile: crate::configuration::Profile,
    ) -> ConfigurationBuilder<crate::configuration::Profile, Debugging, StackSize, Source, Filename>
    {
        ConfigurationBuilder {
            profile,
            debugging: self.debugging,
            stack_size: self.stack_size,
            source: self.source,
            filename: self.filename,
        }
    }
}

impl<Profile, Debugging, StackSize, Source, Filename>
    ConfigurationBuilder<Profile, Debugging, StackSize, Source, Filename>
{
    pub fn debugging(
        self,
        debugging: crate::configuration::Debugging,
    ) -> ConfigurationBuilder<Profile, crate::configuration::Debugging, StackSize, Source, Filename>
    {
        ConfigurationBuilder {
            profile: self.profile,
            debugging,
            stack_size: self.stack_size,
            source: self.source,
            filename: self.filename,
        }
    }
}

impl<Profile, Debugging, StackSize, Source, Filename>
    ConfigurationBuilder<Profile, Debugging, StackSize, Source, Filename>
{
    pub fn stack_size(
        self,
        stack_size: crate::configuration::StackSize,
    ) -> ConfigurationBuilder<Profile, Debugging, crate::configuration::StackSize, Source, Filename>
    {
        ConfigurationBuilder {
            profile: self.profile,
            debugging: self.debugging,
            stack_size,
            source: self.source,
            filename: self.filename,
        }
    }
}

impl<Profile, Debugging, StackSize, Source, Filename>
    ConfigurationBuilder<Profile, Debugging, StackSize, Source, Filename>
{
    pub fn source(
        self,
        source: crate::configuration::Source,
    ) -> ConfigurationBuilder<Profile, Debugging, StackSize, crate::configuration::Source, Filename>
    {
        ConfigurationBuilder {
            profile: self.profile,
            debugging: self.debugging,
            stack_size: self.stack_size,
            source,
            filename: self.filename,
        }
    }
}

impl<Profile, Debugging, StackSize, Source, Filename>
    ConfigurationBuilder<Profile, Debugging, StackSize, Source, Filename>
{
    pub fn filename(
        self,
        filename: crate::configuration::Filename,
    ) -> ConfigurationBuilder<Profile, Debugging, StackSize, Source, crate::configuration::Filename>
    {
        ConfigurationBuilder {
            profile: self.profile,
            debugging: self.debugging,
            stack_size: self.stack_size,
            source: self.source,
            filename,
        }
    }
}
