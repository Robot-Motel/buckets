use crate::errors::BucketError;

/// A trait that defines the interface for all commands in the bucket system.
/// This provides a consistent structure for command execution and initialization.
/// 
/// # Example
/// ```rust
/// use crate::commands::Command;
/// use crate::args::MyCommandArgs;
/// use crate::errors::BucketError;
/// 
/// pub struct MyCommand {
///     args: MyCommandArgs,
/// }
/// 
/// impl Command for MyCommand {
///     type Args = MyCommandArgs;
///     
///     fn new(args: &Self::Args) -> Self {
///         Self { args }
///     }
///     
///     fn execute(&self) -> Result<(), BucketError> {
///         println!("Executing with: {}", self.args.some_field);
///         Ok(())
///     }
/// }
/// ```
pub trait BucketCommand {
    /// The argument type that this command accepts (e.g., InitCommand, CreateCommand, etc.)
    type Args;
    
    /// Creates a new instance of the command with the provided arguments
    fn new(args: &Self::Args) -> Self;
    
    /// Executes the command and returns a Result indicating success or failure
    fn execute(&self) -> Result<(), BucketError>;
}

/// Helper macro to implement the Command trait for a struct.
/// This reduces boilerplate code when creating new commands.
/// 
/// # Usage
/// ```rust
/// use crate::impl_command;
/// use crate::args::MyCommandArgs;
/// 
/// pub struct MyCommand {
///     args: MyCommandArgs,
/// }
/// 
/// impl_command!(MyCommand, MyCommandArgs, {
///     // Implementation of execute method goes here
///     // Access args via self.args
///     println!("Executing with args: {:?}", self.args);
///     Ok(())
/// });
/// ```
#[macro_export]
macro_rules! impl_command {
    ($command_struct:ident, $args_type:ty, $execute_body:block) => {
        impl crate::commands::Command for $command_struct {
            type Args = $args_type;

            fn new(args: Self::Args) -> Self {
                Self { args }
            }

            fn execute(&self) -> Result<(), crate::errors::BucketError> {
                $execute_body
            }
        }
    };
}

/// A dispatcher that provides a centralized way to execute commands using the trait system.
/// This is useful for adding common functionality like logging, metrics, or validation.
/// 
/// # Example
/// ```rust
/// use crate::commands::{CommandDispatcher, init::Init};
/// use crate::args::InitCommand;
/// 
/// // Method 1: Create command instance manually
/// let cmd = Init::new(init_args);
/// CommandDispatcher::execute(cmd)?;
/// 
/// // Method 2: Execute with args (creates instance automatically)
/// CommandDispatcher::execute_with_args::<Init>(init_args)?;
/// ```
pub struct CommandDispatcher;

impl CommandDispatcher {
    /// Execute a command using the trait system with optional pre/post processing
    pub fn execute<T: BucketCommand>(command: T) -> Result<(), BucketError> {
        // Pre-execution hooks can go here (logging, validation, etc.)
        log::debug!("Executing command: {}", std::any::type_name::<T>());
        
        let result = command.execute();
        
        // Post-execution hooks can go here (cleanup, metrics, etc.)
        match &result {
            Ok(_) => log::debug!("Command executed successfully"),
            Err(e) => log::error!("Command failed: {:?}", e),
        }
        
        result
    }
    
    /// Execute a command with arguments, creating the command instance automatically
    pub fn execute_with_args<T: BucketCommand>(args: &T::Args) -> Result<(), BucketError> {
        let command = T::new(args);
        Self::execute(command)
    }
}

pub(crate) mod init;
pub(crate) mod create;
pub(crate) mod commit;
pub(crate) mod restore;
pub(crate) mod rollback;
pub(crate) mod stash;
pub(crate) mod status;
pub(crate) mod history;
pub(crate) mod list;
pub(crate) mod stats;
pub(crate) mod expect;
pub(crate) mod check;
pub(crate) mod link;
pub(crate) mod finalize;
pub mod schema;
