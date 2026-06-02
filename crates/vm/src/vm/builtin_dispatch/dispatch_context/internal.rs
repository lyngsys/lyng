use super::{
    BuiltinInvocation, InternalBuiltinDispatchContext, Value, Vm, VmBuiltinDispatch, VmError,
    errors, eval_builtin,
};

impl InternalBuiltinDispatchContext for VmBuiltinDispatch<'_, '_, '_> {
    type Error = VmError;

    fn function_call_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let target = Vm::require_callable_object(self.agent, invocation.this_value())?;
        let rebound_this = invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined());
        self.vm.call_to_completion(
            self.agent,
            self.host,
            self.registry,
            self.caller,
            target,
            rebound_this,
            invocation.arguments().get(1..).unwrap_or(&[]),
        )
    }

    fn direct_eval_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let callee = invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined());
        let target = Vm::require_callable_object(self.agent, callee)?;
        let caller_realm = self.caller.realm;
        let builtin_eval = self
            .vm
            .builtin_cache
            .builtin_constant(self.agent, caller_realm, eval_builtin())
            .and_then(Value::as_object_ref)
            .ok_or_else(|| VmError::Abrupt(errors::throw_type_error(self.agent)))?;
        if target != builtin_eval {
            return self.vm.call_to_completion(
                self.agent,
                self.host,
                self.registry,
                self.caller,
                target,
                Value::undefined(),
                invocation.arguments().get(1..).unwrap_or(&[]),
            );
        }

        let source = invocation
            .arguments()
            .get(1)
            .copied()
            .unwrap_or(Value::undefined());
        let Some(source_ref) = source.as_string_ref() else {
            return Ok(source);
        };
        if let Some(value) =
            Vm::try_evaluate_regexp_literal_eval_string_ref(self.agent, caller_realm, source_ref)?
        {
            return Ok(value);
        }
        let source_text = self.builtin_value_to_string_text(Value::from_string_ref(source_ref))?;
        let this_override =
            (!invocation.this_value().is_undefined()).then_some(invocation.this_value());
        // Direct eval is compiler-emitted and REAL-frame-only (never the synthetic
        // call_to_completion path): the caller is the live current frame. Eval
        // scope analysis reads the caller's FULL record — including the COLD
        // `parameter_initializer_end_offset` (dynamic_compilation.rs) that
        // `frame_record_for_view` leaves zeroed — so reconstruct the live frame
        // (hot + cold) via `frame()`, not the overlay-only view.
        let caller_record = self
            .vm
            .frame()
            .expect("direct eval runs with the caller as the live frame");
        self.vm.evaluate_direct_eval_source(
            self.agent,
            self.host,
            self.registry,
            caller_record,
            &source_text,
            this_override,
        )
    }

    fn template_to_string_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let value = invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined());
        self.vm
            .template_to_string_builtin(self.agent, self.host, self.registry, self.caller, value)
    }

    fn get_template_object_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.get_template_object_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller,
            invocation.arguments(),
        )
    }

    fn instance_of_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm.instance_of_builtin(
            self.agent,
            self.host,
            self.registry,
            self.caller,
            invocation.arguments(),
        )
    }

    fn define_method_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.define_method_property_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
        )
    }

    fn define_getter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.define_accessor_property_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
            true,
            true,
        )
    }

    fn define_setter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.define_accessor_property_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
            false,
            true,
        )
    }

    fn define_class_getter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.define_accessor_property_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
            true,
            false,
        )
    }

    fn define_class_setter_property_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.define_accessor_property_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
            false,
            false,
        )
    }

    fn define_private_field_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        Vm::define_private_field_builtin(self.agent, caller, invocation.arguments())
    }

    fn private_field_init_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm
            .private_field_init_builtin(self.agent, caller, invocation.arguments())
    }

    fn private_field_get_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.private_field_get_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
        )
    }

    fn private_field_set_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.private_field_set_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
        )
    }

    fn private_has_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm
            .private_has_builtin(self.agent, caller, invocation.arguments())
    }

    fn super_property_get_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.super_property_get_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
        )
    }

    fn super_property_set_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.super_property_set_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
        )
    }

    fn super_base_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm
            .super_base_builtin(self.agent, caller, invocation.arguments())
    }

    fn super_constructor_builtin(
        &mut self,
        _invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.super_constructor_builtin(self.agent, caller)
    }

    fn construct_super_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.construct_super_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
        )
    }

    fn construct_super_spread_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.construct_super_spread_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
        )
    }

    fn construct_super_array_like_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.construct_super_array_like_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
        )
    }

    fn set_function_home_object_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        Vm::set_function_home_object_builtin(self.agent, invocation.arguments())
    }

    fn object_literal_set_prototype_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .object_literal_set_prototype_builtin(self.agent, invocation.arguments())
    }

    fn bind_function_private_env_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        self.vm
            .bind_function_private_env_builtin(self.agent, invocation.arguments())
    }

    fn capture_arrow_context_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm
            .capture_arrow_context_builtin(self.agent, caller, invocation.arguments())
    }

    fn install_instance_field_key_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let caller = self.caller_frame_view();
        self.vm.install_instance_field_key_builtin(
            self.agent,
            self.host,
            self.registry,
            caller,
            invocation.arguments(),
        )
    }

    fn get_instance_field_key_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        Vm::get_instance_field_key_builtin(self.agent, invocation.arguments())
    }

    fn throw_type_error_builtin(
        &mut self,
        _invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let realm = Vm::builtin_realm(self.agent, self.callee_object, self.caller.realm);
        Err(Vm::abrupt_intrinsic_error(
            self.agent,
            realm,
            errors::ErrorKind::Type,
        ))
    }

    fn require_constructor_builtin(
        &mut self,
        invocation: BuiltinInvocation<'_>,
    ) -> Result<Value, Self::Error> {
        let value = invocation
            .arguments()
            .first()
            .copied()
            .unwrap_or(Value::undefined());
        let Some(object) = value.as_object_ref() else {
            let realm = Vm::builtin_realm(self.agent, self.callee_object, self.caller.realm);
            return Err(Vm::abrupt_intrinsic_error(
                self.agent,
                realm,
                errors::ErrorKind::Type,
            ));
        };
        if self.agent.objects().is_constructor(object) {
            return Ok(Value::undefined());
        }

        let realm = Vm::builtin_realm(self.agent, self.callee_object, self.caller.realm);
        Err(Vm::abrupt_intrinsic_error(
            self.agent,
            realm,
            errors::ErrorKind::Type,
        ))
    }
}
