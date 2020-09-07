import React, { useCallback, useState } from 'react';
import { CSSTransition, TransitionGroup } from 'react-transition-group'
import styles from './game.module.css';

const Game = ({ id, userName, game, countdown, ...props }) => {
    const [state, setState] = useState({
        selectedCards: game.player.selectedCards.map((id) => id.toString()),
        useChopsticks: game.player.selectedCards.length > 1,
    });

    const selectCard = useCallback((card) => {
        setState((s) => {
            let cards = s.selectedCards;
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
            <p>
                {game.winner
                    ? <strong>{game.winner} has won the game!</strong>
                    : <strong>Round {game.round}</strong>}
            </p>

            <p>{countdown || '_'}</p>

            <p>Your cards:</p>

            <TransitionGroup className={styles['hand']}>
                {Object.entries(game.player.hand).map(([id, card]) => (
                    <CSSTransition
                        key={id}
                        classNames={{
                            enter: styles['checkable-card-enter'],
                            enterActive: styles['checkable-card-enter-active'],
                            exit: styles['checkable-card-exit'],
                            exitActive: styles['checkable-card-exit-active'],
                        }}
                        timeout={500}>
                        <CheckableCard
                            card={card}
                            checked={state.selectedCards.includes(id)}
                            onChecked={() => selectCard(id)} />
                    </CSSTransition>
                ))}
            </TransitionGroup>

            <div className={styles['confirm']}>
                <button onClick={onSelectedCardsConfirmed}>
                    Confirm selected card
                </button>
            </div>

            <div className={styles['players']}>
                <Player {...game.player} id={userName || 'jakob'} ready={game.player.selectedCards.length > 0} />
                {game.opponents.map((opponent) => <Player key={opponent.id} {...opponent} />)}
            </div>
        </div>
    );
};

const Player = ({ faceUpCards, id, numPoints, numPuddings, ready }) => {
    return (
        <div className={styles['player']} data-ready={ready}>
            <div>
                <div>
                    <strong>{id}</strong>
                </div>
                <div>{numPoints} points</div>
                <div>{numPuddings} puddings</div>
                <div>{ready && <strong>ready</strong>}</div>
            </div>

            <TransitionGroup className={styles['face-up-cards']}>
                {faceUpCards.map((faceUpCard, i) => (
                    <CSSTransition
                        key={i}
                        classNames={{
                            enter: styles['face-up-card-enter'],
                            enterActive: styles['face-up-card-enter-active'],
                            exit: styles['face-up-card-exit'],
                            exitActive: styles['face-up-card-exit-active'],
                        }}
                        timeout={500}>
                        <div className={styles['face-up-card']}>
                            {faceUpCard.kind === 'card'
                                ? <Card {...faceUpCard.card} />
                                : faceUpCard.kind === 'wasabi'
                                    ? <WasabiFaceUpCard {...faceUpCard} />
                                    : null}
                        </div>
                    </CSSTransition>
                ))}
            </TransitionGroup>
        </div >
    );
};

const CheckableCard = ({ card, checked, onChecked }) => {
    return (
        <div className={styles['checkable-card']} data-checked={checked}>
            <label>
                <input
                    className={styles['checkable-card-input']}
                    type="checkbox"
                    checked={checked}
                    onChange={onChecked} />

                <Card {...card} />
            </label>
        </div>
    );
};

const Card = (props) => {
    return (
        <div className={styles['card']} data-kind={props.kind}>
            <div className={styles['card-icon']}></div>

            <CardText {...props} />
        </div>
    );
};

const WasabiFaceUpCard = ({ nigiri }) => {
    return (
        <div className={`${styles['wasabi-nigiri']}`}>
            <Card kind={'wasabi'} />
            <Card kind={'nigiri'} {...nigiri} />
        </div>
    );
};

const CardText = (props) => {
    let text;

    switch (props.kind) {
        case 'makiRolls': {
            const num = props.makiRolls === 'one'
                ? 1
                : props.makiRolls === 'two'
                    ? 2
                    : 3;

            const suffix = num === 1 ? '' : 's';

            text = `${num} maki roll${suffix}`;
            break;
        }

        case 'nigiri': {
            text = `${props.nigiri} nigiri`;
            break;
        }

        default: {
            text = props.kind;
            break;
        }
    }

    return (
        <div className={styles['card-text']}>
            {text}
        </div>
    );
};

export default Game;
