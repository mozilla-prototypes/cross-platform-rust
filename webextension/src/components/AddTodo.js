import React, { Component } from 'react';
import PropTypes from 'prop-types';
import { connect } from 'react-redux';
import { addTodo } from '../actions';
import './AddTodo.css';

class AddTodo extends Component {

  static propTypes = {
    dispatch: PropTypes.func.isRequired,
  }

  state = {
    newTodoName: ''
  }

  onSubmit = (e) => {
    const { dispatch } = this.props;
    e.preventDefault();
    const newTodoName = this.state.newTodoName;
    if (!newTodoName.trim()) {
      return;
    }
    dispatch(addTodo(newTodoName));
    this.setState({newTodoName: ''});
  }

  handleChange = (event) => {
    this.setState({newTodoName: event.target.value});
  }

  render() {
    return (
      <form className="add-todo-form" onSubmit={this.onSubmit}>
        <input className="add-todo-input"
          placeholder="Add a New Todo"
          value={this.state.newTodoName}
          onChange={this.handleChange} />
      </form>
    );
  }
}

export default connect()(AddTodo);
