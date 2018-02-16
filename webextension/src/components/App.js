import React, { Component } from 'react';
import TodoList from './TodoList';
import './App.css';


class App extends Component {

  render() {
    return (
      <div>
        <div className="app">
          <TodoList />
        </div>
      </div>
    );
  }
}

export default App;
