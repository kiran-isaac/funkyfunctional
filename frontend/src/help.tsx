import { useEffect, useRef, useState } from 'react';
import definitionString from './../../definition.md?raw';
import './help.css';
import Markdown from 'markdown-to-jsx'

function definition() {
    const [isVisible, setIsVisible] = useState(true);
    const definitionRef = useRef<HTMLDivElement>(null);

    const handleClickOutside = (event: MouseEvent) => {
        if (definitionRef.current && !definitionRef.current.contains(event.target as Node)) {
            setIsVisible(false);
        }
    };

    useEffect(() => {
        document.addEventListener('mousedown', handleClickOutside);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
        };
    }, []);
    
    return <div id="definition" ref={definitionRef} style={{ display: isVisible ? "block" : "none" }}>
        <button id="definitiondissmis" onClick={() => {
            setIsVisible(!isVisible);
        }}>X</button>
        <Markdown options={{ forceBlock: true }}className="md" children={definitionString}/>
    </div>;
}

export default definition;