/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

#ifndef FLIGHTVARS_UTIL_OPTION_H
#define FLIGHTVARS_UTIL_OPTION_H

#include <memory>

#include <flightvars/util/exception.hpp>

namespace flightvars { namespace util {

FV_DECL_EXCEPTION(option_undefined);

template <class T>
class option {
public:    

    template <class T1> 
    friend class option;

    option() : _data(nullptr) {}

    option(const T& data) : _data(new T(data)) {}

    option(T&& data) : _data(new T(std::move(data))) {}

    option(const option& other) : _data(other.is_defined() ? new T(other.get()) : nullptr) {}

    option(option&& other) : _data(std::move(other._data)) {}

    template <class T1>
    option(const option<T1>& other) : 
        _data(other.is_defined() ? new T(other.get()) : nullptr) {}

    template <class T1>
    option(option<T1>&& other) : _data(std::move(other._data)) {}

    option& operator = (const option& other) {
        _data.reset(other.is_defined() ? new T(other.get()) : nullptr);
        return *this;
    }

    template <class T1>
    option& operator = (const option<T1>& other) {
        _data.reset(other.is_defined() ? new T(other.get()) : nullptr);
        return *this;
    }

    bool valid() const { return is_defined(); }

    bool is_defined() const { return bool(_data); }

    const T& get() const {
        if (is_defined()) { return *_data; }
        else { throw option_undefined("cannot get on not defined option"); }
    }

    T& get() {
        if (is_defined()) { return *_data; }
        else { throw option_undefined("cannot get on not defined option"); }
    }

    const T& get_or_else(const T& other) const {
        return is_defined() ? get() : other;
    }

    void set(const T& data) {
        _data.reset(new T(data));
    }

    T extract() {
        if (is_defined()) {
            auto r = std::move(*_data);
            _data.reset(nullptr);
            return std::move(r);
        } else {
            throw option_undefined("cannot extract on not defined option");
        }
    }

    template <class T1, class Func>
    option<T1> map(Func f) const {
        return is_defined() ? make_some<T1>(f(get())) : make_none<T1>();
    }

    template <class T1, class Func>
    option<T1> fmap(Func f) const {
        return is_defined() ? f(get()) : make_none<T1>();
    }

    template <class T1, class Func>
    option<T1> fold(Func f, const T1& defval) const {
        return is_defined() ? make_some<T1>(f(get())) : make_some(defval);
    }

    template <class Func>
    void for_each(const Func& f) const {
        if (is_defined()) { f(get()); }
    }

private:

    std::unique_ptr<T> _data;
};

template <>
class option<void> {
public:

    template <class T1> 
    friend class option;

    option(bool is_defined = false) : _is_defined(is_defined) {}

    option(const option& other) : _is_defined(other._is_defined) {}

    option& operator = (const option& other) {
        _is_defined = other._is_defined;
        return *this;
    }

    bool valid() const { return is_defined(); }

    bool is_defined() const { return _is_defined; }

    void get() const { 
        if (!is_defined()) {
            throw option_undefined("cannot get on not defined option");
        }
    }

    void set(bool is_defined = true) {
        _is_defined = is_defined;
    }

    void extract() {
        if (!is_defined()) {
            throw option_undefined("cannot extract on not defined option");
        }
        _is_defined = false;
    }

private:

    bool _is_defined;
};

template <class T>
option<T> make_some(const T& value) {
    return option<T>(value);
}

template <class T>
option<T> move_some(T&& value) {
    return option<T>(std::forward<T>(value));
}

inline option<void> make_some() {
    return option<void>();
}

template <class T>
option<T> make_none() {
    return option<T>();
}

}}

#endif
