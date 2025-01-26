import { useEffect, useRef } from 'react';
import definitionString from './../../definition.md?raw';
import './help.css';
import Markdown from 'markdown-to-jsx'

interface DefinitionWindowProps {
    definitionIsVisible: boolean;
    setDefinitionIsVisible: React.Dispatch<React.SetStateAction<boolean>>;
}

const DefinitionWindow: React.FC<DefinitionWindowProps> = ({ definitionIsVisible, setDefinitionIsVisible }) => {
    const definitionRef = useRef<HTMLDivElement>(null);

    const handleClickOutside = (event: MouseEvent) => {
        if (definitionRef.current && !definitionRef.current.contains(event.target as Node)) {
            setDefinitionIsVisible(false);
        }
    };

    useEffect(() => {
        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, []);


    if (!definitionIsVisible) {
        return <></>;
    }
    return <div id="definition" ref={definitionRef}>
        <button id="definitiondissmis" onClick={() => {
            setDefinitionIsVisible(false);
        }}>X</button>
        <Markdown options={{ forceBlock: true }} className="md" children={definitionString} />
    </div>;
}


const DefinitionSpawnButton: React.FC<DefinitionWindowProps> = ({ definitionIsVisible, setDefinitionIsVisible }) => {
    if (!definitionIsVisible) {
        return <button id="definitionspawn" onClick={() => {
        }}>?</button>;
    }
    return <button id="definitionspawn" onClick={() => {
        setDefinitionIsVisible(true);
    }}>?</button>;
}

export {DefinitionWindow, DefinitionSpawnButton};