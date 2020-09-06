import React from 'react';
import { Route, Redirect, Switch, useLocation } from 'react-router-dom';
import { isLoggedIn } from './common/api';
import GameView from './game/GameView';
import HomeView from './home/HomeView';
import LoginView from './login/LoginView';

const App = () => {
    return (
        <div className="app">
            <Switch>
                <Route path="/login">
                    <LoginView />
                </Route>

                <AuthenticatedRoute exact path="/">
                    <HomeView />
                </AuthenticatedRoute>

                <AuthenticatedRoute path="/play/:id">
                    <GameView />
                </AuthenticatedRoute>
            </Switch>
        </div>
    );
};

const AuthenticatedRoute = (props) => {
    const location = useLocation();

    if (!isLoggedIn()) {
        return (
            <Redirect to={{
                pathname: '/login',
                state: { from: location }
            }} />
        );
    }

    return (
        <Route {...props} />
    );
};

export default App;