import React, { useCallback, useMemo, useState } from 'react';
import styles from './login.module.css';

const LoginForm = ({ onLogIn }) => {
    const [userName, setUserName] = useState('');

    const canLogIn = useMemo(() => {
        return userName.length > 0;
    }, [userName]);

    const onUserNameChange = useCallback((e) => {
        setUserName(e.target.value);
    }, []);

    const onLogInClick = useCallback(() => {
        onLogIn(userName);
    }, [onLogIn, userName]);

    const onKeyDown = useCallback((e) => {
        if (e.key === 'Enter' && canLogIn) {
            onLogInClick();
        }
    }, [canLogIn, onLogInClick]);

    return (
        <div className={styles.form}>
            <div>
                <input
                    className={styles.input}
                    id="user-name"
                    type="text"
                    placeholder="Enter user name"
                    value={userName}
                    onKeyDown={onKeyDown}
                    onChange={onUserNameChange}
                    autoFocus />
            </div>

            <button
                className={styles.btn}
                onClick={onLogInClick}
                disabled={!canLogIn}>
                Log in
            </button>
        </div>
    );
};

export default LoginForm;
