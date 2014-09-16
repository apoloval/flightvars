/*
 * FlightVars
 * Copyright (c) 2014 Alvaro Polo
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 */

// Undefine ERROR macro in wingdi.h to avoid clash with log_level::ERROR
#ifdef ERROR
#undef ERROR
#endif

#ifndef FLIGHTVARS_UTIL_LOGGING_H
#define FLIGHTVARS_UTIL_LOGGING_H

#include <boost/core/null_deleter.hpp>
#include <boost/log/attributes/constant.hpp>
#include <boost/log/sinks/text_ostream_backend.hpp>
#include <boost/log/sources/record_ostream.hpp>
#include <boost/log/sources/severity_channel_logger.hpp>
#include <boost/log/utility/setup/common_attributes.hpp>
#include <boost/log/utility/setup/file.hpp>
#include <boost/log/utility/setup/formatter_parser.hpp>

#define LOG_RECORD_FORMAT "%TimeStamp% [%Severity%]: %Message%"

namespace flightvars { namespace util {

namespace logging = boost::log;

enum class log_level { DEBUG, TRACE, INFO, WARN, ERROR, FATAL };

template< typename CharT, typename TraitsT >
inline std::basic_ostream< CharT, TraitsT >& operator<< (
        std::basic_ostream< CharT, TraitsT >& strm, log_level lvl) {
    switch (lvl) {
        case log_level::DEBUG: strm << "DEBUG"; break;
        case log_level::TRACE: strm << "TRACE"; break;
        case log_level::INFO:  strm << "INFO "; break;
        case log_level::WARN:  strm << "WARN "; break;
        case log_level::ERROR: strm << "ERROR"; break;
        case log_level::FATAL: strm << "FATAL"; break;
        default: strm << "UNKNOWN " << static_cast< int >(lvl); break;
    }
    return strm;
}

using logger = logging::sources::severity_channel_logger<log_level, std::string>;

// TODO: have setups for different purposes (testing, production, etc)
void setup_file_logging(const std::string& file_pattern) {
    logging::register_simple_formatter_factory<log_level, char>("Severity");
    logging::add_common_attributes();    

    logging::add_file_log(
        logging::keywords::file_name = file_pattern,
        logging::keywords::format = LOG_RECORD_FORMAT
    );
}

void setup_console_logging() {
    logging::register_simple_formatter_factory<log_level, char>("Severity");
    logging::add_common_attributes();    

    auto core = logging::core::get();
    auto backend = boost::make_shared<logging::sinks::text_ostream_backend>();
    backend->add_stream(
        boost::shared_ptr< std::ostream >(&std::clog, boost::null_deleter()));
    backend->auto_flush(true);

    typedef logging::sinks::synchronous_sink<logging::sinks::text_ostream_backend> sink_t;
    auto frontend = boost::make_shared<sink_t>(backend);
    frontend->set_formatter(logging::parse_formatter(LOG_RECORD_FORMAT));
    core->add_sink(frontend);
}

}}

#endif
