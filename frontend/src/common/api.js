import React from 'react';
import decodeJwt from 'jwt-decode';

export const ApiContext = React.createContext();

export const isLoggedIn = () => {
    return !!localStorage.getItem('token');
};

export const loginAsync = async (userName) => {
    const response = await fetch('/api/login', {
        body: JSON.stringify(userName),
        headers: {
            'Accept': 'application/json',
            'Content-Type': 'application/json'
        },
        method: 'POST',
    });

    if (response.ok) {
        const token = await response.json();

        localStorage.setItem('token', token);
    } else {
        throw new Error(response.statusText);
    }
};

export const logOut = () => {
    localStorage.removeItem('token');
};

export class Api {
    #jwt;
    #decoded;

    constructor() {
        this.#jwt = localStorage.getItem('token');
        this.#decoded = decodeJwt(this.#jwt);
    }

    get userName() {
        return this.#decoded.name;
    }

    async loadGamesAsync() {
        const response = await fetch('/api/games', {
            headers: {
                'Accept': 'application/json',
                'Authorization': `Bearer ${this.#jwt}`,
            },
        });

        if (response.ok) {
            return await response.json();
        }

        throw new Error(response.statusText);
    }

    async createGameAsync(opponents) {
        const response = await fetch('/api/games', {
            body: JSON.stringify(opponents),
            headers: {
                'Accept': 'application/json',
                'Authorization': `Bearer ${this.#jwt}`,
                'Content-Type': 'application/json',
            },
            method: 'POST',
        });

        if (response.ok) {
            const obj = await response.json();

            if (obj.success) {
                return obj.payload;
            }

            throw new Error(obj.error);
        }

        throw new Error(response.statusText);
    }

    async loadGameAsync(gameId) {
        const response = await fetch(`/api/games/${gameId}`, {
            headers: {
                'Accept': 'application/json',
                'Authorization': `Bearer ${this.#jwt}`,
            },
        });

        if (response.ok) {
            return await response.json();
        }

        if (response.status === 404) {
            return null;
        }

        throw new Error(response.statusText);
    }

    async selectCardsAsync(gameId, cards) {
        const response = await fetch(`/api/games/${gameId}`, {
            body: JSON.stringify(cards),
            headers: {
                'Accept': 'application/json',
                'Authorization': `Bearer ${this.#jwt}`,
                'Content-Type': 'application/json',
            },
            method: 'PUT',
        });

        if (response.ok) {
            const obj = await response.json();

            if (obj.success) {
                return;
            }

            throw new Error(obj.error);
        }

        throw new Error(response.statusText);
    }
}
