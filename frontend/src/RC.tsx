interface RCProp {
    i: number;
    multiple: boolean,
    onClick: (rc_index: number) => void;
    from: string;
    to: string;
}

const RC: React.FC<RCProp> = ({ i, multiple, onClick, from, to }) => {
    if (!multiple) {
        return <><button
            className="rc single"
            onClick={() => onClick(i)}
        >
            <div></div>
            <div id="progress">Progress Lazily</div>
            <div></div>
        </button><br /></>;
    } else {
        return <><button
            className="rc multiple"
            onClick={() => onClick(i)}
        >
            <div className="from">
                <pre>{from}</pre>
            </div>
            <div id="rarrow"><p>&rArr;</p></div>
            <div className="to">
                <pre className="to">{to}</pre>
            </div>
        </button><br /></>;        
    }
}

export default RC   