/* eslint-disable jsx-a11y/accessible-emoji */

import React, { useCallback, useMemo } from 'react';
import { NavLink, useHistory } from 'react-router-dom';
import { Api, isLoggedIn, logOut } from './api';
import { MenuTextButton } from './Buttons';
import styles from './layout.module.css';

const Layout = ({ children }) => {
    const api = useMemo(() => isLoggedIn() ? new Api() : null, []);
    const history = useHistory();

    const onLogOutClick = useCallback(() => {
        logOut();
        history.replace('/login');
    }, [history]);

    return (
        <>
            <header>
                <h1>🌸「おすし」🍣</h1>
            </header>

            <nav className={styles['nav']}>
                {api && <>
                    <div className={styles['nav-section']}>
                        <NavLink
                            exact to={'/'}
                            className={styles['nav-link']}
                            activeClassName={styles['active']}>
                            home
                        </NavLink>
                    </div>

                    <div className={styles['nav-section']}>
                        <span>
                            welcome, {api.userName}
                        </span>

                        <MenuTextButton onClick={onLogOutClick}>
                            log out
                        </MenuTextButton>
                    </div>
                </>}
            </nav>

            <main>
                {children}
            </main>
        </>
    );
};

export default Layout;