#[cfg(feature = "expressions")]
use boa_engine::{
    context::ContextBuilder,
    js_string,
    native_function::NativeFunction,
    object::builtins::JsArray,
    property::Attribute,
    Context, JsArgs, JsResult, JsValue, Source,
};

#[cfg(feature = "expressions")]
pub struct ExpressionEvaluator {
    context: Context,
    // We could store current property context here if needed,
    // or just pass it into `evaluate` via global variable updates.
}

#[cfg(feature = "expressions")]
impl ExpressionEvaluator {
    pub fn new() -> Self {
        let mut context = ContextBuilder::default().build().unwrap();

        // Register Global Helpers (AE-style)
        // add(a, b)
        context
            .register_global_callable(
                js_string!("add"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let a = args.get_or_undefined(0);
                    let b = args.get_or_undefined(1);
                    // Basic implementation: if array, add component-wise. Else add scalar.
                    // This is a simplification. AE handles many permutations.
                    helper_add(a, b, context)
                }),
            )
            .unwrap();

        // mul(a, b)
        context
            .register_global_callable(
                js_string!("mul"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let a = args.get_or_undefined(0);
                    let b = args.get_or_undefined(1);
                    helper_mul(a, b, context)
                }),
            )
            .unwrap();

        // sub(a, b)
        context
            .register_global_callable(
                js_string!("sub"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let a = args.get_or_undefined(0);
                    let b = args.get_or_undefined(1);
                    helper_sub(a, b, context)
                }),
            )
            .unwrap();

        // div(a, b)
        context
            .register_global_callable(
                js_string!("div"),
                1,
                NativeFunction::from_fn_ptr(|_this, args, context| {
                    let a = args.get_or_undefined(0);
                    let b = args.get_or_undefined(1);
                    helper_div(a, b, context)
                }),
            )
            .unwrap();

        Self { context }
    }

    pub fn evaluate(
        &mut self,
        script: &str,
        current_value: &JsValue,
        loop_value: &JsValue,
        time: f32,
        _frame_rate: f32,
    ) -> Result<JsValue, String> {
        // Set Globals
        // 'value'
        self.context
            .register_global_property(js_string!("value"), current_value.clone(), Attribute::all())
            .map_err(|e| format!("Failed to register value: {}", e))?;

        // '__loop_value' (internal)
        self.context
            .register_global_property(js_string!("__loop_value"), loop_value.clone(), Attribute::all())
            .map_err(|e| format!("Failed to register loop value: {}", e))?;

        // 'time' (in seconds)
        self.context
            .register_global_property(js_string!("time"), JsValue::new(time), Attribute::all())
            .map_err(|e| format!("Failed to register time: {}", e))?;

        // 'loopOut' implementation that returns pre-calculated cycled value
        self.context
            .register_global_callable(
                js_string!("loopOut"),
                0,
                NativeFunction::from_fn_ptr(|_this, _args, context| {
                     // Return pre-calculated loop value
                     let val = context.global_object().get(js_string!("__loop_value"), context).unwrap_or_default();
                     Ok(val)
                })
            )
             .map_err(|e| format!("Failed to register loopOut: {}", e))?;

        match self.context.eval(Source::from_bytes(script)) {
            Ok(res) => Ok(res),
            Err(e) => Err(format!("Eval error: {}", e)),
        }
    }

    pub fn context(&mut self) -> &mut Context {
        &mut self.context
    }
}

// Helpers

#[cfg(feature = "expressions")]
fn helper_add(a: &JsValue, b: &JsValue, context: &mut Context) -> JsResult<JsValue> {
    // If both are arrays, component-wise add.
    // If one is array, add scalar to components.
    if let (Some(obj_a), Some(obj_b)) = (a.as_object(), b.as_object()) {
        if obj_a.is_array() && obj_b.is_array() {
            let len_a = obj_a.get(js_string!("length"), context)?.to_number(context)? as u64;
            let len_b = obj_b.get(js_string!("length"), context)?.to_number(context)? as u64;
            let len = std::cmp::min(len_a, len_b);
            let mut result = Vec::new();
            for i in 0..len {
                let val_a = obj_a.get(i, context)?.to_number(context)?;
                let val_b = obj_b.get(i, context)?.to_number(context)?;
                result.push(JsValue::new(val_a + val_b));
            }
            return Ok(JsArray::from_iter(result, context).into());
        }
    }
    // Simple scalar fallback or mixed (not fully implemented for MVP)
    let num_a = a.to_number(context)?;
    let num_b = b.to_number(context)?;
    Ok(JsValue::new(num_a + num_b))
}

#[cfg(feature = "expressions")]
fn helper_sub(a: &JsValue, b: &JsValue, context: &mut Context) -> JsResult<JsValue> {
    if let (Some(obj_a), Some(obj_b)) = (a.as_object(), b.as_object()) {
        if obj_a.is_array() && obj_b.is_array() {
            let len_a = obj_a.get(js_string!("length"), context)?.to_number(context)? as u64;
            let len_b = obj_b.get(js_string!("length"), context)?.to_number(context)? as u64;
            let len = std::cmp::min(len_a, len_b);
            let mut result = Vec::new();
            for i in 0..len {
                let val_a = obj_a.get(i, context)?.to_number(context)?;
                let val_b = obj_b.get(i, context)?.to_number(context)?;
                result.push(JsValue::new(val_a - val_b));
            }
            return Ok(JsArray::from_iter(result, context).into());
        }
    }
    let num_a = a.to_number(context)?;
    let num_b = b.to_number(context)?;
    Ok(JsValue::new(num_a - num_b))
}

#[cfg(feature = "expressions")]
fn helper_mul(a: &JsValue, b: &JsValue, context: &mut Context) -> JsResult<JsValue> {
     // Check for Vector * Scalar
    if let Some(obj_a) = a.as_object() {
        if obj_a.is_array() {
            let scalar_b = b.to_number(context)?;
            let len = obj_a.get(js_string!("length"), context)?.to_number(context)? as u64;
            let mut result = Vec::new();
             for i in 0..len {
                let val_a = obj_a.get(i, context)?.to_number(context)?;
                result.push(JsValue::new(val_a * scalar_b));
            }
            return Ok(JsArray::from_iter(result, context).into());
        }
    }
    // Check for Scalar * Vector
    if let Some(obj_b) = b.as_object() {
        if obj_b.is_array() {
            let scalar_a = a.to_number(context)?;
            let len = obj_b.get(js_string!("length"), context)?.to_number(context)? as u64;
            let mut result = Vec::new();
             for i in 0..len {
                let val_b = obj_b.get(i, context)?.to_number(context)?;
                result.push(JsValue::new(scalar_a * val_b));
            }
            return Ok(JsArray::from_iter(result, context).into());
        }
    }

    let num_a = a.to_number(context)?;
    let num_b = b.to_number(context)?;
    Ok(JsValue::new(num_a * num_b))
}

#[cfg(feature = "expressions")]
fn helper_div(a: &JsValue, b: &JsValue, context: &mut Context) -> JsResult<JsValue> {
    // Vector / Scalar
    if let Some(obj_a) = a.as_object() {
        if obj_a.is_array() {
            let scalar_b = b.to_number(context)?;
            if scalar_b == 0.0 { return Ok(JsValue::nan()); }
            let len = obj_a.get(js_string!("length"), context)?.to_number(context)? as u64;
            let mut result = Vec::new();
             for i in 0..len {
                let val_a = obj_a.get(i, context)?.to_number(context)?;
                result.push(JsValue::new(val_a / scalar_b));
            }
            return Ok(JsArray::from_iter(result, context).into());
        }
    }
    let num_a = a.to_number(context)?;
    let num_b = b.to_number(context)?;
    Ok(JsValue::new(num_a / num_b))
}
