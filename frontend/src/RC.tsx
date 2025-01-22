interface RCProp {
    i: number;
    onClick: (rc_index: number) => void;
    from: string;
    to: string;
    laziest: boolean;
}

const RC: React.FC<RCProp> = ({ i, onClick, from, to, laziest }) => {
    return <><button 
        className="rc" 
        onClick={() => onClick(i)} 
        // style={{ color: laziest ? "yellow" : "white" }}
        >
        {from} {"|"} {to}
    </button><br /></>;
}

export default RC   