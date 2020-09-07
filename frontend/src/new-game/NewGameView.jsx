import React, { useCallback, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import { useSubtitle } from '../common/useSubtitle';
import { Api } from '../common/api';
import { TextButton } from '../common/Buttons';
import Layout from '../common/Layout';
import PlayerTextField from '../home/PlayerTextField';
import styles from './new-game.module.css';

export default () => {
    useSubtitle('New Game');

    const api = useMemo(() => new Api(), []);
    const init = useMemo(() => [api.userName, ''], [api.userName]);
    const [players, setPlayers] = useState(init);
    const [error, setError] = useState();
    const [gameId, setGameId] = useState();

    const onChange = useCallback((i, value) => {
        setPlayers(o => [
            ...o.slice(0, i),
            value,
            ...o.slice(i + 1)
        ]);
    }, []);

    const canAddPlayer = useMemo(() => players.length < 5, [players]);

    const onAddPlayerClick = useCallback(() => {
        if (canAddPlayer) {
            setPlayers(ps => [...ps, '']);
        }
    }, [canAddPlayer]);

    const onRemovePlayerClick = useCallback((i) => {
        setPlayers(ps => [
            ...ps.slice(0, i),
            ...ps.slice(i + 1)
        ]);
    }, []);

    const canStartGame = useMemo(() =>
        players.every(o => o.trim())
        && players.length > 1
        && players.length < 6, [players]);


    const onStartGameClick = useCallback(async () => {
        if (canStartGame) {
            setError();
            setGameId();

            const opponents = players.slice(1);

            try {
                const gameId = await api.createGameAsync(opponents);
                setGameId(gameId);
                setPlayers(init);
            } catch (e) {
                setError(e);
            }
        }
    }, [api, canStartGame, init, players]);

    const onKeyDown = useCallback((e) => {
        if (e.key === 'Enter') {
            onStartGameClick();
        }
    }, [onStartGameClick]);

    return (
        <Layout>
            <div className={styles.view}>
                <h2>「さしみ」</h2>
                <h3>New Game</h3>
                <div>Start a new game here!</div>
                <div>For 2-5 players, ages 8+.</div>
                <div>Enter the names of your opponents below:</div>

                <div className={styles['new-game-form']}>
                    <TextButton
                        onClick={onAddPlayerClick}
                        disabled={!canAddPlayer}>
                        Add player
                    </TextButton>

                    {players.map((player, i) => (
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
            </div>
        </Layout>
    );
};
