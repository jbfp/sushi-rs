import React, { useCallback, useEffect } from 'react';
import { useHistory, useLocation } from 'react-router-dom';
import { isLoggedIn, loginAsync } from '../common/api';
import { useSubtitle } from '../common/useSubtitle';
import Layout from '../common/Layout';
import LoginForm from './LoginForm';
import styles from './login.module.css';

const LoginView = () => {
    const history = useHistory();
    const location = useLocation();

    const redirect = useCallback(() => {
        const { from } = location.state || { from: { pathname: '/' } };
        history.replace(from);
    }, [history, location.state]);

    const login = useCallback(async (userName) => {
        await loginAsync(userName);
        redirect();
    }, [redirect]);

    useEffect(() => {
        if (isLoggedIn()) {
            redirect();
        }
    }, [redirect]);

    useSubtitle('Log in');

    return (
        <Layout>
            <div className={styles.view}>
                <h2>「天ぷら」</h2>
                <p>Describe game here</p>
                <span>Please log in to continue</span>
                <LoginForm onLogIn={login} />
            </div>
        </Layout>
    );
};

export default LoginView;
