import * as wasm from 'sfl_wasm_lib'

interface ASTHistoryProps {
    astHistory: wasm.RawASTInfo[];
    rcFromHistory: string[];
    rcToHistory: string[];
}

function colourizeMatches(string: string, toMatch: string, noMatchClassName: string, matchClassName: string , keyPrefix: string): JSX.Element[] {
    const list: JSX.Element[] = [];
    const parts = string.split(toMatch);
    for (let j = 0; j < parts.length; j++) {
        list.push(<span className={noMatchClassName} key={`${keyPrefix}-${j}-nomatch`}>{parts[j]}</span>);
        if (j < parts.length - 1) {
            list.push(<span className={matchClassName} key={`${keyPrefix}-${j}-match`}>{toMatch}</span>);
        }
    }
    return list;
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
    console.log(astStrings, rcFromHistory, rcToHistory);

    const astLIs = [];
    for (let i = astStrings.length - 1; i > 0; i--) {
        let list: JSX.Element[] = [];
        const prev_to_this = rcToHistory[i - 1];
        console.log(i);
        
        // const next_from = rcFromHistory[i];

        let next_from_this = "";
        if (i < rcFromHistory.length - 1) {
            next_from_this = rcFromHistory[i + 1];
        }
        const current = astStrings[i];

        // Get all occurences of rc_from in current, and make them bold
        if (i == astStrings.length - 1) {
            list = colourizeMatches(current, prev_to_this, "", "", `${i}`);
        } else {
            if (prev_to_this.length > next_from_this.length) {
                console.log(1);
                const parts = current.split(prev_to_this);
                for (let j = 0; j < parts.length; j++) {
                    // list.push(<span key={`${i}-${j}`}>{parts[j]}</span>);
                    list = list.concat(colourizeMatches(parts[j], next_from_this, "", "new", `${i}-${j}`));
                    if (j < parts.length - 1) {
                        // list.push(<span className="old" key={`${i}-${j}-old`}>{next_from}</span>);
                        list = list.concat(colourizeMatches(prev_to_this, next_from_this, "new", "new old", `${i}-${j}-new`))
                    }
                }
            } else {
                console.log(2);
                const parts = current.split(next_from_this);
                for (let j = 0; j < parts.length; j++) {
                    // list.push(<span key={`${i}-${j}`}>{parts[j]}</span>);
                    list = list.concat(colourizeMatches(parts[j], prev_to_this, "", "old", `${i}-${j}`));
                    if (j < parts.length - 1) {
                        // list.push(<span className="old" key={`${i}-${j}-old`}>{next_from}</span>);
                        list = list.concat(colourizeMatches(next_from_this, prev_to_this, "old", "new old", `${i}-${j}-new`))
                    }
                }
            }
        }
        astLIs.push(<li className='expr_history' key={i}><pre>{list}</pre></li>);
    }
    astLIs.push(<li className='expr_history' key={0}>{astStrings[0]}</li>);
    // astLIs = astLIs.reverse();   

    return (
        <ul id="ASTHistory">
            {astLIs}
        </ul>
    );
}

export default ASTHistory
