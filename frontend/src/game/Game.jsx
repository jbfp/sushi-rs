import React, { useCallback, useState } from 'react';
import styles from './game.module.css';

const Game = ({ id, game, ...props }) => {
    const [state, setState] = useState({
        selectedCards: game.player.selectedCards.map((id) => id.toString()),
        useChopsticks: game.player.selectedCards.length > 1,
    });

    const selectCard = useCallback((e) => {
        e.persist();

        setState((s) => {
            let cards = s.selectedCards;
            const card = e.target.value;
            const index = cards.indexOf(card);

            if (s.useChopsticks) {
                if (index >= 0) {
                    cards.splice(index, 1);
                } else {
                    cards.push(card);
                }
            } else {
                if (index === 0) {
                    cards = [];
                } else {
                    cards = [card];
                }
            }

            return {
                ...s,
                selectedCards: cards,
            };
        });
    }, []);

    const onSelectedCardsConfirmed = useCallback(() => {
        props.onSelectedCardsConfirmed(state.selectedCards.map(
            id => Number.parseInt(id, 10)));
    }, [props, state.selectedCards]);

    return (
        <div className={styles['game']}>
            <p>Round {game.round}</p>
            <p>Your cards:</p>

            <ul className={styles['card-list']}>
                {Object.entries(game.player.hand).map(([id, card]) => (
                    <li key={id}>
                        <label>
                            <input
                                name="card"
                                type="radio"
                                value={id}
                                checked={state.selectedCards.includes(id)}
                                onChange={selectCard}
                            /> <Card {...card} />
                        </label>
                    </li>
                ))}
            </ul>

            <button onClick={onSelectedCardsConfirmed}>
                Confirm selected card
            </button>

            <code>
                <pre>{JSON.stringify(game, null, 2)}</pre>
            </code>
        </div>
    );
};

const Card = (props) => {
    let text;

    switch (props.kind) {
        case 'makiRolls': {
            const num = props.makiRolls === 'one'
                ? 1
                : props.makiRolls === 'two'
                    ? 2
                    : 3;

            const suffix = num === 1 ? '' : 's';

            text = `${num} Maki Roll${suffix}`;
            break;
        }

        case 'nigiri': {
            text = `${capitalize(props.nigiri)} Nigiri`;
            break;
        }

        default: {
            text = capitalize(props.kind);
            break;
        }
    }

    return (
        <span>{text}</span>
    );
};

const capitalize = (str) => str.replace(/\b\w/g, l => l.toUpperCase());

export default Game;
