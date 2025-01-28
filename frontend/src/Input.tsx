import "./input.css";
import CodeMirror from '@uiw/react-codemirror';
import { SetStateAction, useCallback, useState } from "react";
import { okaidia } from '@uiw/codemirror-theme-okaidia';

const examples = {
    fac: "fac :: Int -> Int\nfac n = if n <= 1 then 1 else n * (fac (n - 1))\nmain = fac 15",
    pair: "second (x, y) = y\nfirst (x, y) = x\npair x y = (x, y)\n\nfac:: Int -> (Int, Int)\nfac n = pair 5 (if n <= 1 then 1 else n * (second (fac (n - 1))))\nmain = second (fac 5)"
}

interface InputProps {
    onRunMultiple: (editorValue: string) => void;
    // onRunSingle: (editorValue: string) => void;
}

function Input({ onRunMultiple }: InputProps) {
    const [editorValue, setEditorValue] = useState(examples.fac);

    // eslint-disable-next-line 
    const editorOnChange = useCallback((val: SetStateAction<string>, _: any) => {
        setEditorValue(val);
    }, []);
    return (
        <>
            <div id="ProgramInput"><CodeMirror
                id="CodeMirrorEditor"
                height="300px"
                width="100%"
                value={editorValue}
                onChange={editorOnChange}
                theme={okaidia}
            /></div>
            {/* <button className="runbutton" id="RunButtonSingle" onClick={() => onRunSingle(editorValue)}>Run 1</button> */}
            <button className="runbutton" id="RunButtonMultiple" onClick={() => onRunMultiple(editorValue)}>Run</button>
        </>
    );
}

export default Input