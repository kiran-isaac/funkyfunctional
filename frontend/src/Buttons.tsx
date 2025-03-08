import { useEffect, useState } from "react";

const eg_programs = import.meta.glob("./../../examples/*", {
    query: '?raw',
    import: 'default',
});

function ProgramDropdown({ setEditorValue }: { setEditorValue: (x: string) => void }) {
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

    const name_options: JSX.Element[] = [];
    egProgramsMap.forEach((_, name) => {
        name_options.push(<option key={name} value={name}>{name}</option>);
    });
    const onChange = () => {
        const e = document.getElementById("program_dropdown") as HTMLSelectElement;
        const val = e?.value;
        if (val == "__local__") {
            setEditorValue(localStorage.getItem("program") || "");
        }
        setEditorValue(egProgramsMap.get(val) || "")
    };

    return <select onChange={onChange} id="program_dropdown">
        <optgroup label="Inbuilt">{name_options}</optgroup>
        <option value="__local__">Local</option>
    </select>
}

interface ButtonsProps {
    handleRun: (programInput: string, multiple: boolean) => void;
    setEditorValue: (x: string) => void;
    editorValue: string;
}

export default function Buttons({ handleRun, setEditorValue, editorValue }: ButtonsProps) {
    return <div id="Buttons">
        <ProgramDropdown setEditorValue={setEditorValue} />
        <button className="runbutton" id="RunButtonSingle" onClick={() => handleRun(editorValue, false)}>Lazy</button>
        <button className="runbutton" id="RunButtonMultiple" onClick={() => handleRun(editorValue, true)}>Free Choice</button>
    </div>
}