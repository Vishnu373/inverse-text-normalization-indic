use rustfst::algorithms::compose::compose;
use rustfst::algorithms::rm_epsilon::rm_epsilon;
use rustfst::algorithms::shortest_path;
use rustfst::prelude::*;

use crate::AppError;

pub(crate) struct Runtime {
    graph: VectorFst<TropicalWeight>,
}

impl Runtime {
    pub(crate) fn load(bytes: &[u8]) -> Result<Self, AppError> {
        let graph =
            VectorFst::load(bytes).map_err(|error| AppError::GraphLoad(error.to_string()))?;
        if graph.start().is_none() {
            return Err(AppError::GraphLoad("graph has no start state".to_owned()));
        }
        Ok(Self { graph })
    }

    pub(crate) fn apply(&self, input: &str) -> Result<String, AppError> {
        let execute = || -> Result<String, Box<dyn std::error::Error>> {
            let mut acceptor = VectorFst::<TropicalWeight>::new();
            let mut state = acceptor.add_state();
            acceptor.set_start(state)?;
            for byte in input.bytes() {
                let next = acceptor.add_state();
                let label = u32::from(byte);
                acceptor.add_tr(state, Tr::new(label, label, TropicalWeight::one(), next))?;
                state = next;
            }
            acceptor.set_final(state, TropicalWeight::one())?;

            let mut lattice: VectorFst<TropicalWeight> =
                compose::<_, VectorFst<_>, VectorFst<_>, _, _, _>(acceptor, &self.graph)?;
            // Remove epsilon-only arcs before selecting the tropical best path.
            rm_epsilon(&mut lattice)?;
            let best: VectorFst<TropicalWeight> = shortest_path(&lattice)?;
            let mut state = best.start().ok_or("input has no accepting path")?;
            let mut output = Vec::new();
            loop {
                if best.is_final(state)? {
                    return Ok(String::from_utf8(output)?);
                }
                let transitions = best.get_trs(state)?;
                let mut transitions = transitions.iter();
                let transition = transitions.next().ok_or("best path is incomplete")?;
                if transitions.next().is_some() {
                    return Err("best path is not linear".into());
                }
                if transition.olabel != 0 {
                    output.push(u8::try_from(transition.olabel)?);
                }
                state = transition.nextstate;
            }
        };
        execute().map_err(|error| AppError::GraphExecution(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use rustfst::prelude::*;

    use super::Runtime;
    use crate::AppError;

    #[test]
    fn rejects_invalid_asset() {
        let error = Runtime::load(b"not a graph").err().expect("invalid graph");
        assert_eq!(error.code(), "GRAPH_LOAD_FAILED");
    }

    #[test]
    fn rejects_serialized_graph_without_start() -> Result<(), Box<dyn std::error::Error>> {
        let mut graph = VectorFst::<TropicalWeight>::new();
        let state = graph.add_state();
        graph.set_final(state, TropicalWeight::one())?;
        let mut bytes = Vec::new();
        graph.store(&mut bytes)?;
        assert!(VectorFst::<TropicalWeight>::load(&bytes)?.start().is_none());

        let error = Runtime::load(&bytes).err().expect("missing start state");
        assert!(matches!(error, AppError::GraphLoad(_)));
        assert_eq!(error.code(), "GRAPH_LOAD_FAILED");
        Ok(())
    }

    #[test]
    fn rejects_input_without_accepting_path() -> Result<(), Box<dyn std::error::Error>> {
        let mut graph = VectorFst::<TropicalWeight>::new();
        let start = graph.add_state();
        let end = graph.add_state();
        graph.set_start(start)?;
        graph.add_tr(start, Tr::new(b'a'.into(), b'b'.into(), 0.0, end))?;
        graph.set_final(end, TropicalWeight::one())?;

        let error = Runtime { graph }.apply("z").expect_err("unmatched input");
        assert!(matches!(error, AppError::GraphExecution(_)));
        assert_eq!(error.code(), "GRAPH_EXECUTION_FAILED");
        Ok(())
    }

    #[test]
    fn rejects_non_byte_and_invalid_utf8_output() -> Result<(), Box<dyn std::error::Error>> {
        for output_label in [256, 255] {
            let mut graph = VectorFst::<TropicalWeight>::new();
            let start = graph.add_state();
            let end = graph.add_state();
            graph.set_start(start)?;
            graph.add_tr(start, Tr::new(b'a'.into(), output_label, 0.0, end))?;
            graph.set_final(end, TropicalWeight::one())?;

            let error = Runtime { graph }.apply("a").expect_err("invalid output");
            assert!(matches!(error, AppError::GraphExecution(_)));
            assert_eq!(error.code(), "GRAPH_EXECUTION_FAILED", "{output_label}");
        }
        Ok(())
    }

    #[test]
    fn selects_lower_cost_path_with_epsilon_and_unicode() -> Result<(), Box<dyn std::error::Error>>
    {
        let mut graph = VectorFst::<TropicalWeight>::new();
        let start = graph.add_state();
        graph.set_start(start)?;
        let input = "\u{d15}";
        let expected = "\u{d67}";

        // Insert the expensive path first; both paths consume the same UTF-8 bytes.
        for (cost, output) in [(5.0, "wrong"), (1.0, expected)] {
            let mut state = graph.add_state();
            graph.add_tr(start, Tr::new(0, 0, cost, state))?;
            for byte in input.bytes() {
                let next = graph.add_state();
                graph.add_tr(state, Tr::new(byte.into(), 0, 0.0, next))?;
                state = next;
            }
            for byte in output.bytes() {
                let next = graph.add_state();
                graph.add_tr(state, Tr::new(0, byte.into(), 0.0, next))?;
                state = next;
            }
            graph.set_final(state, TropicalWeight::one())?;
        }

        assert_eq!(Runtime { graph }.apply(input)?, expected);
        Ok(())
    }
}
