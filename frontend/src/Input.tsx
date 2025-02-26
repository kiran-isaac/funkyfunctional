import "./input.css";
import CodeMirror from '@uiw/react-codemirror';
import { SetStateAction, useCallback, useEffect, useState } from "react";
import { okaidia } from '@uiw/codemirror-theme-okaidia';

const eg_programs = import.meta.glob("./../../examples/*", {
    query: '?raw',
    import: 'default',
});


function ProgramDropdown({ inbuiltPrograms, setEditorValue }: { inbuiltPrograms: Map<string, string>, setEditorValue: (x : string) => void}) {    
    const name_options : JSX.Element[] = [];
    inbuiltPrograms.forEach((_, name) => {
        name_options.push(<option value={name}>{name}</option>);
    });
    const onChange = () => {
        const e = document.getElementById("program_dropdown") as HTMLSelectElement;
        const val = e?.value;
        if (val == "__local__") {
            setEditorValue(localStorage.getItem("program") || "");
        }
        setEditorValue(inbuiltPrograms.get(val) || "")
    };

    return <select onChange={onChange} id="program_dropdown">
        <optgroup label="Inbuilt">{name_options}</optgroup>
        <option value="__local__">Local</option>
    </select>
}

interface InputProps {
    onRunMultiple: (editorValue: string) => void;
    onRunSingle: (editorValue: string) => void;
}

function Input({ onRunMultiple, onRunSingle }: InputProps) {
    const [editorValue, setEditorValue] = useState("");
    const [egProgramsMap, setEgProgramsMap] = useState<Map<string, string>>(new Map());

    useEffect(() => {
        const loadPrograms = async () => {
            const eg_programs_map: Map<string, string> = new Map();
            for (const path in eg_programs) {
                const program = await eg_programs[path]();
                
                const eg_name = path.split('\\').pop()?.split('/').pop()?.replace(".sfl", "");
                if (eg_name == undefined) { return; }
                eg_programs_map.set(eg_name, program as string)
            }

            setEgProgramsMap(eg_programs_map)
        };
        loadPrograms();
    }, []);

    // eslint-disable-next-line
    const editorOnChange = useCallback((val: SetStateAction<string>, _: any) => {
        // override local storage 
        localStorage.setItem("program", val.toString());
        const e = document.getElementById("program_dropdown") as HTMLSelectElement;
        e.value = "__local__"
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
            <ProgramDropdown inbuiltPrograms={egProgramsMap} setEditorValue={setEditorValue}/>
            <button className="runbutton" id="RunButtonSingle" onClick={() => onRunSingle(editorValue)}>Lazy</button>
            <button className="runbutton" id="RunButtonMultiple" onClick={() => onRunMultiple(editorValue)}>Free Choice</button>
            <hr/>
        </>
    );
}

export default Input