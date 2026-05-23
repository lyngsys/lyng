use lyng_js_env::Agent;
use lyng_js_types::{EmbeddingFunctionId, PropertyKey, Value};
use lyng_js_vm::{
    EmbeddingFunctionContext, EmbeddingFunctionMetadata, EmbeddingInvocation,
    RealmExtensionInstallation, RealmExtensionProvider, VmError,
};
use std::io::{self, Write};

const CLI_PRINT_RAW: u32 = 1;

pub struct CliRealmExtension;

const fn cli_print_entry() -> EmbeddingFunctionId {
    EmbeddingFunctionId::from_raw(CLI_PRINT_RAW)
        .expect("CLI embedding function ids should stay non-zero")
}

fn cli_property_key(agent: &mut Agent, text: &str) -> PropertyKey {
    PropertyKey::from_atom(agent.atoms_mut().intern_collectible(text))
}

impl RealmExtensionProvider for CliRealmExtension {
    fn embedding_function_metadata(
        &self,
        entry: EmbeddingFunctionId,
    ) -> Option<EmbeddingFunctionMetadata> {
        if entry == cli_print_entry() {
            return Some(EmbeddingFunctionMetadata::new("print", 1, false, false));
        }
        None
    }

    fn install_realm_extensions(
        &self,
        installation: &mut RealmExtensionInstallation<'_>,
    ) -> Result<(), VmError> {
        let print_key = cli_property_key(installation.agent(), "print");
        let _ = installation.define_function_property(
            installation.global_object(),
            print_key,
            cli_print_entry(),
            true,
            false,
            true,
        )?;
        Ok(())
    }

    fn call_embedding_function(
        &self,
        context: &mut EmbeddingFunctionContext<'_>,
        entry: EmbeddingFunctionId,
        invocation: EmbeddingInvocation<'_>,
    ) -> Result<Value, VmError> {
        if entry != cli_print_entry() {
            return Err(VmError::MissingEmbeddingFunction(entry));
        }

        let mut parts = Vec::with_capacity(invocation.arguments().len());
        for value in invocation.arguments() {
            parts.push(context.value_to_string_text(*value)?);
        }
        let _ = writeln!(io::stdout(), "{}", parts.join(" "));
        Ok(Value::undefined())
    }
}
