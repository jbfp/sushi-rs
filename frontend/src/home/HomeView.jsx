import React, { useCallback, useEffect, useMemo, useState } from 'react';
import { Link } from 'react-router-dom';
import { useSubtitle } from '../common/useSubtitle';
import { Api } from '../common/api';
import Layout from '../common/Layout';
import GameList from './GameList';
import styles from './home.module.css';

export default () => {
    useSubtitle('Home');

    const api = useMemo(() => new Api(), []);
    const [games, setGames] = useState();

    const loadGames = useCallback(async () => {
        await api.loadGamesAsync().then(setGames);
    }, [api]);

    useEffect(() => {
        loadGames();
    }, [loadGames]);

    return (
        <Layout>
            <div className={styles.view}>
                <h2 className={styles.header}>
                    Sushi Game「にぎり」
                </h2>

                <p className={styles.description}>
                    In the super-fast sushi card game Sushi Go!, you are eating at a sushi restaurant and trying to grab the best combination of sushi dishes as they whiz by. Score points for collecting the most sushi rolls or making a full set of sashimi. Dip your favorite nigiri in wasabi to triple its value! And once you've eaten it all, finish your meal with all the pudding you've got! But be careful which sushi you allow your friends to take; it might be just what they need to beat you!
                </p>

                <h3>Games</h3>

                <Link to={'/new-game'}>
                    <button>New game</button>
                </Link>

                <GameList games={games} />
            </div>
        </Layout>
    );
};