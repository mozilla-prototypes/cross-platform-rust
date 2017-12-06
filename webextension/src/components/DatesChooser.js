import React, { Component } from 'react';
import PropTypes from 'prop-types';
import './DatesChooser.css';

// Convert a date object to a yyyy-mm-dd string.
function convertToYYYYMMDD(date) {
  if (!date) {
    return '';
  }
  return new Date(date).toISOString().substr(0, 10);
}

// Inverse operation
function convertToDate(dateStr) {
  if (!dateStr) {
    return null;
  }
  return new Date(dateStr).getTime();
}

class DatesChooser extends Component {

  static propTypes = {
    dueDate: PropTypes.number,
    completionDate: PropTypes.number,
    onTodoChangeDueDate: PropTypes.func.isRequired,
    onTodoChangeCompletionDate: PropTypes.func.isRequired,
  }

  constructor(props) {
    super(props);
    this.state = { newDueDate: convertToYYYYMMDD(this.props.dueDate),
      newCompletionDate: convertToYYYYMMDD(this.props.completionDate) };
  }

  handleDueDateChange = (e) => {
    const newDueDate = e.target.value;
    this.setState({ newDueDate });
    this.props.onTodoChangeDueDate(convertToDate(newDueDate));
  }

  handleCompletionDateChange = (e) => {
    const newCompletionDate = e.target.value;
    if (newCompletionDate !== '' && new Date(newCompletionDate) > new Date()) {
      return;
    }
    this.setState({ newCompletionDate });
    this.props.onTodoChangeCompletionDate(convertToDate(newCompletionDate));
  }

  onSubmit = (e) => {
    e.preventDefault();
  }

  render() {
    return (
      <div className="todo-dates-chooser">
        <span>Due Date</span>
        <input type="date" value={this.state.newDueDate}
          onChange={this.handleDueDateChange} />
        <span>Completion Date</span>
        <input type="date" value={this.state.newCompletionDate}
          onChange={this.handleCompletionDateChange} />
      </div>
    );
  }
}

export default DatesChooser;
