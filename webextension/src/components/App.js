import React, { Component } from 'react';
import TodoList from './TodoList';
import LabelsEditor from './LabelsEditor';
import './App.css';


class App extends Component {

  state = {
    displayLabelsEditor: false
  }

  toggleLabelEditor = () => {
    this.setState({displayLabelsEditor: !this.state.displayLabelsEditor});
  }

  render() {
    return (
      <div>
        <div className="app">
          <LabelsEditor isOpened={this.state.displayLabelsEditor} />
          <TodoList />
        </div>
        <button className="edit-labels-button" onClick={this.toggleLabelEditor}>Edit Labels</button>
      </div>
    );
  }
}

export default App;
