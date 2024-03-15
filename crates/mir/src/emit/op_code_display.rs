use std::io;

use rokugo_backend_common::{FunctionId, ValueId};
use rokugo_common::color::{ColorSpec, ColoredDisplay};
use termcolor::{Color, WriteColor};

use super::op_code::{MirInstruction, MirInstructionData, MirInstructionMeta};

const COLOR_MEMORY: ColorSpec = ColorSpec {
    fg: Some(Color::Blue),
    intense: true,
};
const COLOR_CONTROL_FLOW: ColorSpec = COLOR_MEMORY;
const COLOR_META: ColorSpec = ColorSpec {
    fg: Some(Color::Black),
    ..ColorSpec::default()
};

const COLOR_VALUE_ID: ColorSpec = ColorSpec {
    fg: Some(Color::Yellow),
    ..ColorSpec::default()
};
const COLOR_FUNCTION_ID: ColorSpec = ColorSpec {
    fg: Some(Color::Green),
    ..ColorSpec::default()
};

impl ColoredDisplay for MirInstruction<'_> {
    fn fmt_with_color(&self, f: &mut dyn WriteColor) -> io::Result<()> {
        self.meta.fmt_with_color(f)?;
        self.data.fmt_with_color(f)?;
        Ok(())
    }
}

fn write_result(f: &mut dyn WriteColor, result: &ValueId) -> io::Result<()> {
    f.set_color(&COLOR_VALUE_ID.into())?;
    write!(f, "{}", result)?;
    f.reset()?;
    write!(f, " = ")
}

fn write_value_id(f: &mut dyn WriteColor, value_id: &ValueId) -> io::Result<()> {
    f.set_color(&COLOR_VALUE_ID.into())?;
    write!(f, "{}", value_id)
}

fn write_function_id(f: &mut dyn WriteColor, function_id: &FunctionId) -> io::Result<()> {
    f.set_color(&COLOR_FUNCTION_ID.into())?;
    write!(f, "{}", function_id)
}

impl ColoredDisplay for MirInstructionData<'_> {
    fn fmt_with_color(&self, f: &mut dyn WriteColor) -> io::Result<()> {
        match self {
            // ! Memory
            MirInstructionData::DefineInt32(result, value) => {
                write_result(f, result)?;
                f.set_color(&COLOR_MEMORY.into())?;
                write!(f, "DefineInt32 ")?;
                f.reset()?;
                write!(f, "{}", value)?;
            }
            // ! Control flow
            MirInstructionData::ReturnValue(value) => {
                f.set_color(&COLOR_CONTROL_FLOW.into())?;
                write!(f, "ReturnValue ")?;
                write_value_id(f, value)?;
            }
            MirInstructionData::Call(result, function_id, arguments) => {
                write_result(f, result)?;
                f.set_color(&COLOR_CONTROL_FLOW.into())?;
                write!(f, "Call ")?;
                write_function_id(f, function_id)?;
                for argument in arguments.iter() {
                    write!(f, " ")?;
                    write_value_id(f, argument)?;
                }
            }
        }

        writeln!(f)?;
        Ok(())
    }
}

impl ColoredDisplay for MirInstructionMeta {
    fn fmt_with_color(&self, f: &mut dyn WriteColor) -> io::Result<()> {
        f.set_color(&COLOR_META.into())?;

        if let Some(span) = &self.span {
            writeln!(f, "@MetaSpan: {}..{}", span.start, span.end)?;
        }
        Ok(())
    }
}
