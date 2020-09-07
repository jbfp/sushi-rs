import React from 'react';
import Game from './Game';

class GameHost extends React.Component {
    constructor(props) {
        super(props);

        this.countdownId = null;
        this.sse = new EventSource(`/api/games/${props.gameId}/stream`);

        this.state = {
            time: null,
        };
    }

    componentDidMount() {
        this.sse.addEventListener('cardsselected', this.handleCardsSelected);
        this.sse.addEventListener('countdownstarted', this.handleCountdownStarted);
        this.sse.addEventListener('countdowncancelled', this.handleCountdownCancelled);
        this.sse.addEventListener('turnover', this.handleTurnOver);
        this.sse.addEventListener('roundover', this.handleRoundOver);
        this.sse.addEventListener('gameover', this.handleGameOver);
    }

    handleCardsSelected = ({ data }) => {
        const userId = JSON.parse(data);

        console.log(`player ${userId} selected cards`);

        this.props.onPlayerIsReady(userId);
    };

    handleCountdownStarted = ({ data }) => {
        const ms = JSON.parse(data);

        console.log(`countdown in ${ms} ms`);

        this.clearCountdown();

        this.setState({ time: ms / 1000 });

        this.countdownId = window.setInterval(() => {
            this.setState(s => ({ time: s.time - 1 }));
        }, 1000);
    };

    handleCountdownCancelled = () => {
        console.log('cancelled countdown');
        this.clearCountdown();
    };

    handleTurnOver = () => {
        window.setTimeout(() => {
            this.clearCountdown();
            this.props.onTurnOver();
        }, 1000);
    };

    handleRoundOver = ({ data }) => {
        this.props.onRoundOver(JSON.parse(data));
    };

    handleGameOver = ({ data }) => {
        this.props.onGameOver(JSON.parse(data));
    };

    clearCountdown = () => {
        window.clearInterval(this.countdownId);
        this.countdownId = null;
        this.setState({ time: null });
    };

    componentWillUnmount() {
        this.clearCountdown();
        this.sse.close();
    }

    render() {
        return (
            <Game {...this.props} countdown={this.state.time} />
        );
    }
}

export default GameHost;
