import * as wasm from 'sfl_wasm_lib'
import './App.css'
import RC from './RC';

interface RCsProps {
    ast: wasm.RawASTInfo;
    setAstString: (s: string) => void;
};

const RCs = ({ ast, setAstString }: RCsProps) => {
        const rcs = wasm.get_one_redex(ast);
    if (wasm.get_rcs_len(rcs) === 0) {
        return <ul></ul>;
    }

    const rc_callback = (rc_index: number) => {
        ast = wasm.pick_rc_and_free(ast, rcs, rc_index);
        setAstString(wasm.to_string(ast));
        RCs(ast);
    };

    // const rc_elems = [<div key={0}><button className="rc" id="laziest" onClick={() => rc_callback(laziest)}>Laziest</button><br /></div>];
    const rc_elems = [];

    for (let i = 0; i < wasm.get_rcs_len(rcs); i++) {
        const from_string = wasm.get_rcs_from(rcs, i);
        const to_string = wasm.get_rcs_to(rcs, i);
        rc_elems.push(<RC key={i + 1} i={i} onClick={rc_callback} from={from_string} to={to_string} />);
    }

    return <ul>{rc_elems}</ul>;
}

