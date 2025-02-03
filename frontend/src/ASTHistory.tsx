import * as wasm from 'sfl_wasm_lib'

interface ASTHistoryProps {
    astHistory: wasm.RawASTInfo[];
    rcFromHistory: string[];
    rcToHistory: string[];
}

const ASTHistory = ({ astHistory, rcFromHistory, rcToHistory }: ASTHistoryProps) => {
    const astStrings = [];
    for (let i = 0; i < astHistory.length; i++) {
        astStrings.push(wasm.main_to_string(astHistory[i]) + "\n");
    }

    if (astStrings.length === 0) {
        return <></>;
    }

    // Get diffs between each string
    // console.log(astStrings, rcFromHistory, rcToHistory);

    const astLIs = [];
    for (let i = astStrings.length - 1; i >= 0; i--) {
        const list: JSX.Element[] = [];
        const prev_to_this = rcToHistory[i - 1];

        const current = astStrings[i];

        // Get all occurences of rc_from in current, and make them bold
        if (i == astStrings.length - 1) {
            const parts = current.split(prev_to_this);
            for (let j = 0; j < parts.length; j++) {
                list.push(<span key={`${i}-${j}`}>{parts[j]}</span>);
                if (j < parts.length - 1) {
                    list.push(<span className="new" key={`${i}-${j}-new`}>{prev_to_this}</span>);
                }
            }
        } else {
            const next_from_this = rcFromHistory[i];
            const parts = current.split(next_from_this);
            
            for (let j = 0; j < parts.length; j++) {
                list.push(<span key={`${i}-${j}`}>{parts[j]}</span>);
                if (j < parts.length - 1) {
                    list.push(<span className="old" key={`${i}-${j}-old`}>{next_from_this}</span>);
                }
            }
        }

        astLIs.push(<li className='expr_history' key={i}>{list}<br /></li>);
    }
    // astLIs = astLIs.reverse();   

    return (
        <ul id="ASTHistory">
            {astLIs}
        </ul>
    );
}

export default ASTHistory
