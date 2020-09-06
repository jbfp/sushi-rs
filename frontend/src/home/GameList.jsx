import React from 'react';
import { Link } from 'react-router-dom';

const GameList = ({ games }) => {
    return (
        <div className="game-list">
            <h3>Games</h3>

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