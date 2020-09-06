import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { Api } from '../common/api';
import { useSubtitle } from '../common/useSubtitle';
import Layout from '../common/Layout';
import NewGameForm from './NewGameForm';
import GameList from './GameList';
import styles from './home.module.css';

const HomeView = () => {
    const api = useMemo(() => new Api(), []);
    const [games, setGames] = useState();

    const loadGames = useCallback(async () => {
        await api.loadGamesAsync().then(setGames);
    }, [api]);

    const onStartGame = useCallback(async (opponents) => {
        const gameId = await api.createGameAsync(opponents);
        loadGames();
        return gameId;
    }, [api, loadGames]);

    useSubtitle('Home');

    useEffect(() => {
        loadGames();
    }, [loadGames]);

    return (
        <Layout>
            <div className={styles.view}>
                <h2 className={styles.header}>
                    「にぎり」
                </h2>

                <p className={styles.description}>
                    describe game here
                </p>

                <NewGameForm
                    userName={api.userName}
                    onStartGame={onStartGame} />

                <GameList games={games} />
            </div>
        </Layout>
    );
};

export default HomeView;