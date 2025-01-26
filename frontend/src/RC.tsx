interface RCProp {
    i: number;
    onClick: (rc_index: number) => void;
    from: string;
    to: string;
}

const RC: React.FC<RCProp> = ({ i, onClick, from, to }) => {
    return <><button
        className="rc"
        onClick={() => onClick(i)}
    >
        <div className="from">
            <pre>{from}</pre>
        </div> 
        <div id="rarrow"><pre>&rArr;</pre></div> 
        <div className="to">
            <pre className="to">{to}</pre>
        </div>
    </button><br /></>;
}

export default RC   