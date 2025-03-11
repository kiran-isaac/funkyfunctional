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
        } else if (i == astStrings.length - 2) {
            const next_from_this = rcFromHistory[i];
            const parts = current.split(next_from_this);

            for (let j = 0; j < parts.length; j++) {
                list.push(<span key={`${i}-${j}`}>{parts[j]}</span>);
                if (j < parts.length - 1) {
                    list.push(<span className="old" key={`${i}-${j}-old`}>{next_from_this}</span>);
                }
            }
        } else {
            list.push(<span key={i}>{current}</span>);
        }

        astLIs.push(<li className='expr_history' key={i}><pre>{list}</pre></li>);
    }
    // astLIs = astLIs.reverse();   

    return (
        <div id="ASTHistoryWrapper">
            <table id="ASTHistory">
                <tbody>
                    {astLIs.map((li, index) => (
                        <tr key={astLIs.length - index - 1} className={index == 0 ? 'top' : ''}>
                            <td className='index'>{astLIs.length - index - 1}</td>
                            <td className='ast'>{li}</td>
                            {/* {index < astLIs.length - 1 && <hr />} */}
                        </tr>
                    ))}
                </tbody>
            </table>
        </div>
    );
}

export default ASTHistory
