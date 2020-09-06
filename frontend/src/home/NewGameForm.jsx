import React, { useCallback, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import { TextButton } from '../common/Buttons';
import PlayerTextField from './PlayerTextField';
import styles from './home.module.css';

const NewGameForm = ({ userName, onStartGame }) => {
    const init = useMemo(() => [userName, ''], [userName]);
    const [opponents, setOpponents] = useState(init);
    const [error, setError] = useState();
    const [gameId, setGameId] = useState();

    const onChange = useCallback((i, value) => {
        setOpponents(o => [
            ...o.slice(0, i),
            value,
            ...o.slice(i + 1)
        ]);
    }, []);

    const canAddPlayer = useMemo(() => opponents.length < 5, [opponents]);

    const onAddPlayerClick = useCallback(() => {
        if (canAddPlayer) {
            setOpponents(opponents => [...opponents, '']);
        }
    }, [canAddPlayer]);

    const onRemovePlayerClick = useCallback((i) => {
        setOpponents(opponents => [
            ...opponents.slice(0, i),
            ...opponents.slice(i + 1)
        ]);
    }, []);

    const canStartGame = useMemo(() =>
        opponents.every(o => o.trim())
        && opponents.length > 1
        && opponents.length < 6, [opponents]);

    const onStartGameClick = useCallback(async () => {
        if (canStartGame) {
            setError();
            setGameId();

            try {
                setGameId(await onStartGame(opponents.slice(1)));
                setOpponents(init);
            } catch (e) {
                setError(e);
            }
        }
    }, [canStartGame, init, opponents, onStartGame]);

    const onKeyDown = useCallback((e) => {
        if (e.key === 'Enter') {
            onStartGameClick();
        }
    }, [onStartGameClick]);

    console.log(styles);

    return (
        <div className={styles['new-game-form']}>
            <h3>New Game</h3>

            <p>Create new game here!</p>

            <TextButton
                onClick={onAddPlayerClick}
                disabled={!canAddPlayer}>
                Add player
            </TextButton>

            {opponents.map((player, i) => (
                <div key={i} className={styles['player-form']}>
                    <PlayerTextField
                        index={i}
                        value={player}
                        disabled={i === 0}
                        onKeyDown={onKeyDown}
                        onChange={(e) => onChange(i, e.target.value)} />

                    {i > 1 && (
                        <TextButton
                            className={styles['remove-link']}
                            onClick={() => onRemovePlayerClick(i)}>
                            Remove
                        </TextButton>
                    )}
                </div>
            ))}

            <button
                onClick={onStartGameClick}
                disabled={!canStartGame}>
                Start game
            </button>

            {error && <div>{error.message}</div>}

            {gameId && (
                <p>
                    Your game is ready <Link to={`/play/${gameId}`}>here</Link>
                </p>
            )}
        </div>
    );
};

export default NewGameForm;
