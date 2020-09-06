import React from 'react';

const PlayerTextField = ({
    index, value, disabled,
    onKeyDown, onChange, onBlur
}) => {
    return (
        <input
            className="player-text-field"
            type="text"
            placeholder={`Player ${index + 1}*`}
            value={value}
            onKeyDown={onKeyDown}
            onChange={onChange}
            onBlur={onBlur}
            disabled={disabled} />
    );
};

export default PlayerTextField;
