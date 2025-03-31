import { useContext, useEffect, useState } from "react";
import { SettingsContext } from "./SettingsProvider";

const eg_programs = import.meta.glob("./../../examples/*", {
    query: '?raw',
    import: 'default',
});

const default_program = "square_sum";

function ProgramDropdown({ setEditorValue, userProgramsMap, setUserProgramsMap }: { setEditorValue: (x: string) => void, userProgramsMap: object, setUserProgramsMap: (x: object) => void }) {
    const [egProgramsMap, setEgProgramsMap] = useState<Map<string, string>>(new Map());

    const selectedProgram = localStorage.getItem("selectedProgram") || default_program;

    useEffect(() => {
        const loadPrograms = async () => {
            const eg_programs_map: Map<string, string> = new Map();
            for (const path in eg_programs) {
                const program = await eg_programs[path]();

                const eg_name = path.split('\\').pop()?.split('/').pop()?.replace(".sfl", "");
                if (eg_name == undefined) { continue; }

                // Skip programs that have the same value syntax as the local storage key
                if (eg_name.startsWith("__local__") && eg_name.endsWith("__")) { continue; }
                eg_programs_map.set(eg_name, program as string)
            }

            if (!eg_programs_map.has(selectedProgram)) {
                console.error("Default program not found in eg_programs_map");
            } else {
                setEditorValue(eg_programs_map.get(selectedProgram) || "");
            }

            setEgProgramsMap(eg_programs_map);
        };
        loadPrograms()
    }, [setEditorValue, selectedProgram]);

    const eg_name_options: JSX.Element[] = [];
    const user_name_options: JSX.Element[] = [];
    egProgramsMap.forEach((_, name) => { eg_name_options.push(<option key={name} value={name}>{name}</option>); });
    Object.keys(userProgramsMap).forEach((name) => {
        user_name_options.push(<option key={name} value={"__local__" + name}>{name}</option>);
    });

    const onChange = () => {
        const e = document.getElementById("program_dropdown") as HTMLSelectElement;
        const val = e?.value;
        localStorage.setItem("selectedProgram", val);
        if (val.startsWith("__local__")) {
            const localVal = val.replace("__local__", "");
            let new_val = userProgramsMap[localVal];
            setEditorValue(new_val);
        } else {
            setEditorValue(egProgramsMap.get(val) || "")
        }
    };

    return <select onChange={onChange} id="program_dropdown" value={selectedProgram} >
        <optgroup label="Inbuilt">{eg_name_options}</optgroup>
        <optgroup label="Local">{user_name_options}</optgroup>
    </select>
}

interface ButtonsProps {
    handleRun: (programInput: string, multiple: boolean) => void;
    setEditorValue: (x: string) => void;
    editorValue: string;
    settingsIsVisible: boolean;
    setErrorString: (x: string) => void;
    setSettingsIsVisible: React.Dispatch<React.SetStateAction<boolean>>;
}

interface SaveButtonProps { 
    name: string, 
    setName: (x: string) => void, 
    editorValue: string, 
    setUserProgramsMap: (x: object) => void 
    setErrorString: (x: string) => void
}

function SaveButton({ name, setName, editorValue, setUserProgramsMap, setErrorString }: SaveButtonProps) {
    return <div id="save_button">
        <button id="" onClick={() => {
            setUserProgramsMap((prev: object) => {
                if (name === "") {
                    setErrorString("Please enter a name for the program");
                    return prev;
                }
                const newMap = { ...prev };
                newMap[name] = editorValue;
                localStorage.setItem("user_programs", JSON.stringify(newMap));
                return newMap;
            });
        }}>
            Save As
        </button>
        <textarea id="save_name" value={name} onChange={(e) => {
            const newName = e.target.value;
            setName(newName);
        }}></textarea>
    </div>
}

export default function Buttons({ handleRun, setEditorValue, editorValue, settingsIsVisible, setSettingsIsVisible, setErrorString }: ButtonsProps) {
    const [name, setName] = useState<string>("");

    let user_programs = localStorage.getItem("user_programs");
    if (user_programs == null) {
        user_programs = JSON.stringify({});
    }

    let loadedMap = JSON.parse(user_programs);
    if (loadedMap == null) {
        loadedMap = new Map();
    }

    // Check if the loadedMap is empty
    if (Object.keys(loadedMap).length === 0) {
        loadedMap = { "untitled": "shmungus" };
        localStorage.setItem("user_programs", JSON.stringify(loadedMap));
    }

    const [userProgramsMap, setUserProgramsMap] = useState<object>(loadedMap);

    return <div id="Buttons">
        <ProgramDropdown setEditorValue={setEditorValue} userProgramsMap={userProgramsMap} setUserProgramsMap={setUserProgramsMap} />
        <SaveButton setErrorString={setErrorString} setName={setName} name={name} editorValue={editorValue} setUserProgramsMap={setUserProgramsMap} />
        <button onClick={() => { setSettingsIsVisible(!settingsIsVisible); console.log("BLungus") }}>Settings</button>
        <button className="runbutton" id="RunButtonSingle" onClick={() => handleRun(editorValue, false)}>Lazy</button>
        <button className="runbutton" id="RunButtonMultiple" onClick={() => handleRun(editorValue, true)}>Free Choice</button>
    </div>
}