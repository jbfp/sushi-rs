import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { useHistory, useParams } from 'react-router-dom';
import { Api } from '../common/api';
import { useSubtitle } from '../common/useSubtitle';
import Layout from '../common/Layout';
import GameHost from './GameHost';

const GameView = () => {
    const { id } = useParams();
    const api = useMemo(() => new Api(), []);
    const history = useHistory();
    const [game, setGame] = useState();

    const onSelectedCardsConfirmed = useCallback((cards) => {
        return api.selectCardsAsync(id, cards)
    }, [api, id]);

    const loadGame = useCallback(async () => {
        console.log('loading game...');

        const game = await api.loadGameAsync(id);

        if (game === null) {
            history.replace('/');
        } else {
            setGame(game);
        }
    }, [api, history, id]);

    useEffect(() => {
        loadGame();
    }, [loadGame]);

    return (
        <Layout>
            <div className="game-view">
                {game && <MutableGame
                    gameId={id}
                    refresh={loadGame}
                    game={game}
                    onSelectedCardsConfirmed={onSelectedCardsConfirmed}
                />}
            </div>
        </Layout>
    );
};

const MutableGame = ({ game: g, refresh, ...props }) => {
    const [game, setGame] = useState(g);

    const handlePlayerIsReady = useCallback((userId) => {
        setGame(game => {
            const opponents = game.opponents;
            const index = opponents.findIndex((p) => p.id === userId);

            if (index < 0) {
                refresh();
                return game;
            }

            return {
                ...game, opponents: [
                    ...opponents.slice(0, index),
                    { ...opponents[index], ready: true },
                    ...opponents.slice(index + 1)
                ]
            };
        });
    }, [refresh]);

    const handleTurnOver = useCallback(() => {
        refresh();
    }, [refresh]);

    const handleRoundOver = useCallback(() => {

    }, []);

    const handleGameOver = useCallback((winner) => {
        setGame(g => ({ ...g, winner }));
    }, []);

    useEffect(() => {
        setGame(g);
    }, [g]);

    useSubtitle('Play');

    return (
        <GameHost
            game={game}
            onPlayerIsReady={handlePlayerIsReady}
            onTurnOver={handleTurnOver}
            onRoundOver={handleRoundOver}
            onGameOver={handleGameOver}
            {...props} />
    );
};

export default GameView;
