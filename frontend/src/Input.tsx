import "./input.css";
import CodeMirror from '@uiw/react-codemirror';
import { SetStateAction, useCallback, useState } from "react";
import { okaidia } from '@uiw/codemirror-theme-okaidia';
import starterProgram from './../../starter_program.sfl?raw';


interface InputProps {
    onRunMultiple: (editorValue: string) => void;
    onRunSingle: (editorValue: string) => void;
}

function Input({ onRunMultiple, onRunSingle }: InputProps) {
    let program = localStorage.getItem("program");
    if (program === null) {
        program = starterProgram;
    }    
    const [editorValue, setEditorValue] = useState(program);

    // eslint-disable-next-line
    const editorOnChange = useCallback((val: SetStateAction<string>, _: any) => {
        // override local storage 
        localStorage.setItem("program", val.toString());
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
            <button className="runbutton" id="RunButtonSingle" onClick={() => onRunSingle(editorValue)}>Lazy</button>
            <button className="runbutton" id="RunButtonMultiple" onClick={() => onRunMultiple(editorValue)}>Free Choice</button>
        </>
    );
}

export default Input