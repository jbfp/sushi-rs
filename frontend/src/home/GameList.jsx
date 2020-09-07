import React from 'react';
import { Link } from 'react-router-dom';
import styles from './home.module.css';

const GameList = ({ games }) => {
    return (
        <div className={styles['game-list']}>
            {games
                ? games.length > 0 ? (
                    games.map(({ id, players }) => (
                        <div key={id}>
                            <Link to={`/play/${id}`}>
                                {players.join(' vs ')}
                            </Link>
                        </div>
                    ))
                ) : <p>No games available bro</p>
                : <p>Loading...</p>}
        </div>
    );
};

export default GameList;