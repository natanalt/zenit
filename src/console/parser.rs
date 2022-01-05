use log::error;
use crate::engine::Engine;
use super::{ConsoleValueKind, ConsoleValue, Console};

impl Console {
    pub fn run_command(&mut self, engine: &mut Engine, command: &str) {
        let mut split = command.split(' ');
        let command_name = split.next().unwrap().to_string();

        let mut next_parameter = |expected: ConsoleValueKind,
                                  allow_long_unquoted_string: bool|
         -> Option<ConsoleValue> {
            if let Some(next) = split.next() {
                match expected {
                    ConsoleValueKind::Bool => {
                        next.parse::<bool>().map(|x| ConsoleValue::Bool(x)).ok()
                    }
                    ConsoleValueKind::Int32 => {
                        next.parse::<i32>().map(|x| ConsoleValue::Int32(x)).ok()
                    }
                    ConsoleValueKind::Float32 => {
                        next.parse::<f32>().map(|x| ConsoleValue::Float32(x)).ok()
                    }
                    ConsoleValueKind::String => {
                        if !next.starts_with('"') {
                            if allow_long_unquoted_string {
                                let mut result = next.to_string();
                                while let Some(next) = split.next() {
                                    result.push(' ');
                                    result.push_str(next);
                                }
                                Some(result)
                            } else {
                                Some(next.to_string())
                            }
                        } else if next.starts_with('"') && next.ends_with('"') {
                            if next.len() == 1 {
                                // Just the ", which technically passes the conditional
                                None
                            } else {
                                // Get the correct string length, in a way respecting Unicode
                                // Requires a reenumeration, but I don't quite care, commands aren't hot
                                // code paths.
                                let len = next.chars().count();
                                Some(next.chars().skip(1).take(len - 2).collect())
                            }
                        } else {
                            let mut result = next.chars().skip(1).collect::<String>();
                            let mut properly_finished = false;
                            while let Some(next) = split.next() {
                                result.push(' ');
                                if next.ends_with('"') {
                                    let len = next.chars().count();
                                    result.push_str(&next.chars().take(len - 1).collect::<String>());
                                    properly_finished = true;
                                    break;
                                } else {
                                    result.push_str(next);
                                }   
                            }
                            
                            if properly_finished {
                                Some(result)
                            } else {
                                None
                            }
                        }.map(|x| ConsoleValue::String(x))
                    }
                }
            } else {
                None
            }
        };

        if let Some(convar) = self.convars.get_mut(&command_name) {
            let expected = convar.default_value.kind();
            let new_value = next_parameter(expected, true);
            if new_value.is_none() {
                error!("{:?} value expected", expected);
                return;
            } else if split.next().is_some() {
                error!("{:?} value expected; too many parameters given", expected);
                return;
            }
            convar.update(engine, new_value.unwrap());
        } else if let Some(command) = self.commands.get_mut(&command_name) {
            let mut params = Vec::new();
            for (info, i) in command.parameters.iter().zip(0..) {
                if let Some(value) = next_parameter(info.kind, false) {
                    params.push(value);
                } else {
                    error!("{:?} value expected at parameter {}", info.kind, i + 1);
                    return;
                }
            }

            if split.next().is_some() {
                error!("Too many parameters given");
                return;
            }

            (command.callback)(engine, &params).unwrap_or_else(|err| {
                error!("An error occurred while executing this command");
                error!("Details: {}", err);
            });
        } else {
            error!(
                "Command `{}` not found, run `help` to get a listing.",
                command_name
            );
        }
    }
}
